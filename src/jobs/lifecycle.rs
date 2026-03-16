use crate::db::DbClient;
use crate::models::{AllSetting, Inbound};
use crate::repositories::setting::SettingOps;
use crate::services::tgbot;
use chrono::Utc;
use log::{error, info};

use std::time::Duration;
use tokio::time::sleep;

/// Configuration for the lifecycle job
#[derive(Debug, Clone)]
pub struct LifecycleConfig {
    pub check_interval: Duration,
}

impl Default for LifecycleConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(60),
        }
    }
}

/// Start the user lifecycle background job (expiration & traffic limits)
pub async fn start_lifecycle_job(db: DbClient, config: LifecycleConfig) {
    info!(
        "Starting user lifecycle job (interval: {:?})",
        config.check_interval
    );

    loop {
        sleep(config.check_interval).await;

        match perform_lifecycle_check(&db).await {
            Ok(count) => {
                if count > 0 {
                    info!("Lifecycle job disabled {} clients", count);
                }
            }
            Err(e) => {
                error!("Failed to perform lifecycle check: {}", e);
            }
        }
    }
}

/// Check all clients and disable those who have expired or exceeded traffic limits
async fn perform_lifecycle_check(db: &DbClient) -> anyhow::Result<usize> {
    #[cfg(feature = "server")]
    {
        // 1. Fetch System Settings (for Telegram notifications)
        let settings = <AllSetting as SettingOps>::get(db)
            .await
            .unwrap_or(None)
            .unwrap_or_default();

        // 2. Fetch Inbounds
        let mut inbounds: Vec<Inbound> = db.client.select("inbound").await?;
        let mut total_disabled = 0;
        let now = Utc::now().timestamp_millis();

        for inbound in &mut inbounds {
            let mut inbound_modified = false;

            // Use the helper method to access mutable clients for supported protocols
            if let Some(clients) = inbound.settings.clients_mut() {
                for client in clients {
                    if !client.enable {
                        continue;
                    }

                    let mut reason = None;

                    // Check Expiration
                    if client.expiry_time > 0 && client.expiry_time < now {
                        reason = Some("expired");
                    }
                    // Check Traffic Limit
                    else if client.total_flow_limit > 0 {
                        let total_used = client.up.saturating_add(client.down); // bytes
                        let limit_bytes = client.total_flow_limit * 1024 * 1024 * 1024; // GB to bytes

                        if total_used as u64 >= limit_bytes {
                            reason = Some("exceeded traffic limit");
                        }
                    }

                    // Disable if needed
                    if let Some(r) = reason {
                        client.enable = false;
                        inbound_modified = true;
                        total_disabled += 1;

                        let email = client.email.as_deref().unwrap_or("unknown");
                        info!("Client {} {}. Disabling.", email, r);

                        // Send Telegram Notification
                        if settings.tg_bot_enable
                            && ((r == "expired" && settings.tg_notify_expiry)
                                || (r == "exceeded traffic limit" && settings.tg_notify_traffic))
                        {
                            if let (Some(token), Some(chat_id)) =
                                (&settings.tg_bot_token, &settings.tg_bot_chat_id)
                            {
                                let msg = format!(
                                    "🚫 <b>Client Disabled</b>\n\nUser: {}\nReason: {}\nInbound: {}",
                                    email, r, inbound.remark
                                );
                                // Spawning task to avoid blocking loop
                                let token = token.clone();
                                let chat_id = chat_id.clone();
                                tokio::spawn(async move {
                                    if let Err(e) =
                                        tgbot::send_message(&token, &chat_id, &msg).await
                                    {
                                        error!(
                                            "Failed to send Telegram disable notification: {}",
                                            e
                                        );
                                    }
                                });
                            }
                        }
                    }
                }
            }

            if inbound_modified {
                if let Some(id) = &inbound.id {
                    // Update the inbound in DB
                    // Note: Update using ID needs to be specific in SurrealDB
                    let _updated: Option<Inbound> = db
                        .client
                        .update((id.tb.clone(), id.id.to_string()))
                        .content(inbound.clone())
                        .await?;
                }
            }
        }

        Ok(total_disabled)
    }

    #[cfg(not(feature = "server"))]
    {
        Ok(0)
    }
}
