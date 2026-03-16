// src/jobs/billing.rs
use crate::AppState;
use crate::models::Inbound;
use crate::services::orchestrator::CoreOrchestrator;
use chrono::Utc;
use log::{error, info};
use std::sync::Arc;
use std::time::Duration;

/// Starts the billing background job.
/// Checks every 60 seconds for expired or over-limit users and updates their status.
pub async fn start_billing_job(state: Arc<AppState>) {
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    info!("Billing Job started.");

    loop {
        interval.tick().await;
        check_users(&state).await;
    }
}

async fn check_users(state: &Arc<AppState>) {
    // Fetch all inbounds
    // We fetch as Vec<Inbound>
    let mut inbounds: Vec<Inbound> = match state.db.client.select("inbound").await {
        Ok(res) => res,
        Err(e) => {
            error!("Billing Job: Failed to fetch inbounds: {}", e);
            return;
        }
    };

    let now = Utc::now().timestamp();

    for inbound in &mut inbounds {
        let mut modified = false;
        let inbound_tag = inbound.tag.clone();

        // Use clients_mut helper to iterate over clients regardless of protocol
        if let Some(clients) = inbound.settings.clients_mut() {
            for client in clients.iter_mut() {
                // Check expiry
                // expiry_time > 0 means it has an expiry.
                // If expiry_time == 0, it never expires.
                let is_expired = client.expiry_time > 0 && now > client.expiry_time;

                // Check traffic
                // If total_gb == 0, usually implies unlimited unless specified otherwise.
                // We assume 0 is unlimited.
                // total_gb is in GB, up/down is in bytes.
                let limit_bytes = if client.total_flow_limit > 0 {
                    client.total_flow_limit.saturating_mul(1024 * 1024 * 1024)
                } else {
                    0
                };

                let is_traffic_limit_reached =
                    limit_bytes > 0 && (client.up + client.down) >= limit_bytes as i64;

                // Determine effective status
                let should_be_enabled = !is_expired && !is_traffic_limit_reached;

                // State transition check
                if client.enable != should_be_enabled {
                    let email = client.email.clone().unwrap_or_default();
                    let uuid = client.id.clone().unwrap_or_default();
                    let mut sync_success = false;

                    // 1. Attempt to Sync with Core FIRST (Transactional Safety)
                    if !email.is_empty() && !uuid.is_empty() {
                        if should_be_enabled {
                            // Re-enabling
                            match state
                                .orchestrator
                                .add_user(&inbound_tag, &email, &uuid, 0)
                                .await
                            {
                                Ok(_) => {
                                    info!("Billing Job: Re-enabled user {} in Core.", email);
                                    sync_success = true;
                                }
                                Err(e) => {
                                    error!(
                                        "Billing Job: Failed to re-enable user {}: {}. Will retry next tick.",
                                        email, e
                                    );
                                    // We prevent DB update so we retry later
                                }
                            }
                        } else {
                            // Disabling
                            match state.orchestrator.remove_user(&inbound_tag, &email).await {
                                Ok(_) => {
                                    info!("Billing Job: Disabled user {} in Core.", email);
                                    sync_success = true;
                                }
                                Err(e) => {
                                    error!(
                                        "Billing Job: Failed to disable user {}: {}. Will retry next tick.",
                                        email, e
                                    );
                                }
                            }
                        }
                    } else {
                        // If no email/uuid, we assume it's valid to update DB even if we can't sync
                        // (e.g. malformed user that we want to disable anyway)
                        sync_success = true;
                    }

                    // 2. Commit to DB only if Sync succeeded (or wasn't needed)
                    if sync_success {
                        info!(
                            "Billing: User {} (email: {:?}) status change: {} -> {}. Reason: Expired={}, Limit={}",
                            client.id.as_deref().unwrap_or("?"),
                            client.email,
                            client.enable,
                            should_be_enabled,
                            is_expired,
                            is_traffic_limit_reached
                        );
                        client.enable = should_be_enabled;
                        modified = true;
                    }
                }
            }
        }

        if modified {
            if let Some(record) = &inbound.id {
                // Update the inbound record
                // Assumption: record.id.to_string() gives the ID part compatible with update()
                let record_id = record.id.to_string();
                let _: Result<Option<Inbound>, _> = state
                    .db
                    .client
                    .update(("inbound", record_id))
                    .content(inbound.clone())
                    .await;
            }
        }
    }
}
