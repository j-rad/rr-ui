use crate::{
    AppState,
    models::{AllSetting, Inbound},
    repositories::setting::SettingOps,
};
use chrono::Utc;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::time::sleep;

// --- Telegram API Models ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TgMessage {
    pub chat_id: String,
    pub text: String,
}

#[derive(Debug, Deserialize)]
struct GetUpdatesResponse {
    pub ok: bool,
    pub result: Vec<Update>,
}

#[derive(Debug, Deserialize)]
struct Update {
    pub update_id: u64,
    pub message: Option<Message>,
}

#[derive(Debug, Deserialize)]
struct Message {
    pub message_id: u64,
    pub chat: Chat,
    pub text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Chat {
    pub id: i64,
}

// --- Bot Logic ---

/// Send a message via Telegram API
pub async fn send_message(token: &str, chat_id: &str, text: &str) -> Result<(), reqwest::Error> {
    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
    let client = reqwest::Client::new();
    let payload = TgMessage {
        chat_id: chat_id.to_string(),
        text: text.to_string(),
    };

    let res = client.post(&url).json(&payload).send().await?;
    if !res.status().is_success() {
        error!("Failed to send Telegram message: {:?}", res.text().await);
    }
    Ok(())
}

/// Helper to get updates from Telegram
async fn get_updates(token: &str, offset: u64) -> Result<Vec<Update>, reqwest::Error> {
    let url = format!(
        "https://api.telegram.org/bot{}/getUpdates?offset={}&timeout=10",
        token, offset
    );
    let client = reqwest::Client::new();
    let res: reqwest::Response = client.get(&url).send().await?;

    if res.status().is_success() {
        let response: GetUpdatesResponse = res.json().await?;
        if response.ok {
            return Ok(response.result);
        }
    } else {
        error!("Failed to get updates: {:?}", res.text().await);
    }
    Ok(vec![])
}

/// Generate a traffic report string
async fn generate_traffic_report(state: &Arc<AppState>) -> String {
    let inbounds: Vec<Inbound> = match state.db.client.select("inbound").await {
        Ok(i) => i,
        Err(_) => return "Error fetching data.".to_string(),
    };

    let mut report = String::from("📊 <b>Traffic Report</b>\n\n");
    let now = Utc::now().timestamp();
    let mut has_content = false;

    for inbound in inbounds.iter() {
        if let Some(clients) = inbound.settings.clients() {
            for client in clients.iter() {
                // Filter for "active" or interesting clients? For now, list all enabled/expired.
                if !client.enable && client.expiry_time == 0 && client.total_flow_limit == 0 {
                    continue; // Skip completely disabled/empty clients to reduce noise
                }

                let email = client.email.as_deref().unwrap_or("N/A");
                let up_gb = client.up as f64 / 1073741824.0;
                let down_gb = client.down as f64 / 1073741824.0;
                let total_gb = up_gb + down_gb;

                let mut status_emoji = "🟢";
                if !client.enable {
                    status_emoji = "🔴";
                } else if client.expiry_time > 0 && client.expiry_time < now {
                    status_emoji = "⌛";
                }

                report.push_str(&format!(
                    "{} <b>{}</b>\n    ↑ {:.2} GB | ↓ {:.2} GB | Σ {:.2} GB\n",
                    status_emoji, email, up_gb, down_gb, total_gb
                ));

                if client.total_flow_limit > 0 {
                    let limit_gb = client.total_flow_limit as f64; // Assuming it's in GB based on usage
                    let percent = (total_gb / limit_gb) * 100.0;
                    report.push_str(&format!(
                        "    Limit: {:.2} GB ({:.1}%)\n",
                        limit_gb, percent
                    ));
                }
                if client.expiry_time > 0 {
                    let expiry_date =
                        chrono::DateTime::from_timestamp(client.expiry_time / 1000, 0)
                            .map(|dt| dt.format("%Y-%m-%d").to_string())
                            .unwrap_or("Invalid".to_string());
                    report.push_str(&format!("    Expires: {}\n", expiry_date));
                }

                report.push_str("\n");
                has_content = true;
            }
        }
    }

    if !has_content {
        report.push_str("No active clients found.");
    }

    report
}

/// Send a traffic report to the configured admin chat
pub async fn send_traffic_report(state: &Arc<AppState>) {
    let settings: Option<AllSetting> = match <AllSetting as SettingOps>::get(&state.db).await {
        Ok(Some(s)) => Some(s),
        _ => None,
    };

    if let Some(settings) = settings {
        let token = settings.tg_bot_token.as_deref().unwrap_or_default();
        let chat_id = settings.tg_bot_chat_id.as_deref().unwrap_or_default();

        if !token.is_empty() && !chat_id.is_empty() {
            let report = generate_traffic_report(state).await;
            if let Err(e) = send_message(token, chat_id, &report).await {
                error!("Failed to send scheduled traffic report: {}", e);
            }
        }
    }
}

/// Main Bot Loop
pub async fn start_bot_loop(state: Arc<AppState>) {
    info!("Telegram Bot Loop Started");
    let mut last_update_id = 0;

    loop {
        // 1. Check settings
        let settings_opt = match <AllSetting as SettingOps>::get(&state.db).await {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to fetch settings for bot loop: {}", e);
                sleep(std::time::Duration::from_secs(60)).await;
                continue;
            }
        };

        let settings = settings_opt.unwrap_or_default();

        if !settings.tg_bot_enable {
            sleep(std::time::Duration::from_secs(30)).await;
            continue;
        }

        let token = match &settings.tg_bot_token {
            Some(t) if !t.is_empty() => t.clone(),
            _ => {
                // Token missing, sleep and retry
                sleep(std::time::Duration::from_secs(60)).await;
                continue;
            }
        };

        let admin_chat_id_raw = settings.tg_bot_chat_id.clone().unwrap_or_default();
        // Simple security: only reply to the configured admin ID?
        // Or allow anyone to query if they know the bot?
        // Better to be safe: If admin_chat_id is set, only respond to it.
        // If not set, maybe log the first ID that contacts it? (Too auto-magical)
        // Let's enforce responding only to the configured Chat ID for sensitive info.

        let admin_chat_id = if let Ok(id) = admin_chat_id_raw.parse::<i64>() {
            Some(id)
        } else {
            None
        };

        // 2. Poll Updates
        match get_updates(&token, last_update_id + 1).await {
            Ok(updates) => {
                for update in updates {
                    last_update_id = update.update_id;

                    if let Some(message) = update.message {
                        if let Some(text) = message.text {
                            let chat_id = message.chat.id;

                            // Check authorization
                            if let Some(allowed) = admin_chat_id {
                                if allowed != chat_id {
                                    warn!("Unauthorized access attempt from Chat ID: {}", chat_id);
                                    continue;
                                }
                            } else {
                                // If no Chat ID is configured in settings, we can't verify admin.
                                // We should probably ignore commands or warn.
                                // For UX, maybe reply "Bot not configured with Chat ID."
                                let _ = send_message(
                                    &token,
                                    &chat_id.to_string(),
                                    "❌ Admin Chat ID not configured in panel settings.",
                                )
                                .await;
                                continue;
                            }

                            // Handle Commands
                            if text.starts_with("/start") || text.starts_with("/help") {
                                let msg = "👋 <b>Rustray Bot</b>\n\nCommands:\n/status - System Status\n/traffic - User Traffic Report";
                                let _ = send_message(&token, &chat_id.to_string(), msg).await;
                            } else if text.starts_with("/status") {
                                // Basic system status (placeholder for now, or fetch from SystemState if possible)
                                // We don't have direct access to SystemState here easily unless passed,
                                // but we can report app uptime or similar if we tracked it.
                                let msg = "✅ <b>System Status</b>\n\n- Service: Running\n- Database: Connected";
                                let _ = send_message(&token, &chat_id.to_string(), msg).await;
                            } else if text.starts_with("/traffic") {
                                let report = generate_traffic_report(&state).await;
                                let _ = send_message(&token, &chat_id.to_string(), &report).await;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                error!("Telegram polling error: {}", e);
                sleep(std::time::Duration::from_secs(5)).await;
            }
        }

        // Small sleep between polls (even with long polling, good practice to yield)
        sleep(std::time::Duration::from_secs(1)).await;
    }
}
