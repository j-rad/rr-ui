// src/services/native_alerts.rs
//! Native Alerts System
//!
//! Telegram and Discord notifications with rate limiting

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Alert message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub alert_id: String,
    pub timestamp: i64,
    pub severity: AlertSeverity,
    pub title: String,
    pub message: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl AlertSeverity {
    pub fn emoji(&self) -> &'static str {
        match self {
            AlertSeverity::Info => "ℹ️",
            AlertSeverity::Warning => "⚠️",
            AlertSeverity::Error => "❌",
            AlertSeverity::Critical => "🚨",
        }
    }
}

/// Telegram notifier
#[cfg(feature = "server")]
pub struct TelegramNotifier {
    bot_token: String,
    chat_id: String,
    rate_limiter: RateLimiter,
    client: reqwest::Client,
}

#[cfg(feature = "server")]
impl TelegramNotifier {
    pub fn new(bot_token: String, chat_id: String) -> Self {
        Self {
            bot_token,
            chat_id,
            rate_limiter: RateLimiter::new(Duration::from_secs(60), 20),
            client: reqwest::Client::new(),
        }
    }

    /// Send alert to Telegram
    pub async fn send_alert(&mut self, alert: &Alert) -> Result<(), String> {
        if !self.rate_limiter.allow() {
            return Err("Rate limit exceeded".to_string());
        }

        let message = self.format_message(alert);
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);

        let params = serde_json::json!({
            "chat_id": self.chat_id,
            "text": message,
            "parse_mode": "Markdown",
            "disable_web_page_preview": true,
        });

        match self.client.post(&url).json(&params).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(())
                } else {
                    Err(format!("Telegram API error: {}", response.status()))
                }
            }
            Err(e) => Err(format!("Request failed: {}", e)),
        }
    }

    fn format_message(&self, alert: &Alert) -> String {
        let mut msg = format!(
            "{} *{}*\n\n{}\n\n",
            alert.severity.emoji(),
            alert.title,
            alert.message
        );

        if !alert.metadata.is_empty() {
            msg.push_str("*Details:*\n");
            for (key, value) in &alert.metadata {
                msg.push_str(&format!("• {}: `{}`\n", key, value));
            }
        }

        msg.push_str(&format!("\n_Alert ID: {}_", alert.alert_id));

        msg
    }
}

/// Discord notifier
#[cfg(feature = "server")]
pub struct DiscordNotifier {
    webhook_url: String,
    rate_limiter: RateLimiter,
    client: reqwest::Client,
}

#[cfg(feature = "server")]
impl DiscordNotifier {
    pub fn new(webhook_url: String) -> Self {
        Self {
            webhook_url,
            rate_limiter: RateLimiter::new(Duration::from_secs(60), 30),
            client: reqwest::Client::new(),
        }
    }

    /// Send alert to Discord
    pub async fn send_alert(&mut self, alert: &Alert) -> Result<(), String> {
        if !self.rate_limiter.allow() {
            return Err("Rate limit exceeded".to_string());
        }

        let embed = self.create_embed(alert);
        let payload = serde_json::json!({
            "embeds": [embed]
        });

        match self
            .client
            .post(&self.webhook_url)
            .json(&payload)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(())
                } else {
                    Err(format!("Discord webhook error: {}", response.status()))
                }
            }
            Err(e) => Err(format!("Request failed: {}", e)),
        }
    }

    fn create_embed(&self, alert: &Alert) -> serde_json::Value {
        let color = match alert.severity {
            AlertSeverity::Info => 0x3498db,     // Blue
            AlertSeverity::Warning => 0xf39c12,  // Orange
            AlertSeverity::Error => 0xe74c3c,    // Red
            AlertSeverity::Critical => 0x992d22, // Dark red
        };

        let mut fields = Vec::new();
        for (key, value) in &alert.metadata {
            fields.push(serde_json::json!({
                "name": key,
                "value": format!("`{}`", value),
                "inline": true,
            }));
        }

        serde_json::json!({
            "title": format!("{} {}", alert.severity.emoji(), alert.title),
            "description": alert.message,
            "color": color,
            "fields": fields,
            "footer": {
                "text": format!("Alert ID: {}", alert.alert_id)
            },
            "timestamp": chrono::DateTime::from_timestamp(alert.timestamp, 0)
                .unwrap_or_else(|| chrono::Utc::now())
                .to_rfc3339(),
        })
    }
}

/// Rate limiter with exponential backoff
pub struct RateLimiter {
    window: Duration,
    max_requests: usize,
    requests: Vec<Instant>,
    backoff_until: Option<Instant>,
}

impl RateLimiter {
    pub fn new(window: Duration, max_requests: usize) -> Self {
        Self {
            window,
            max_requests,
            requests: Vec::new(),
            backoff_until: None,
        }
    }

    /// Check if request is allowed
    pub fn allow(&mut self) -> bool {
        let now = Instant::now();

        // Check if in backoff period
        if let Some(backoff) = self.backoff_until {
            if now < backoff {
                return false;
            } else {
                self.backoff_until = None;
            }
        }

        // Remove old requests outside window
        self.requests
            .retain(|&req_time| now.duration_since(req_time) < self.window);

        // Check if under limit
        if self.requests.len() < self.max_requests {
            self.requests.push(now);
            true
        } else {
            // Trigger exponential backoff
            let backoff_duration = Duration::from_secs(
                60 * 2_u64.pow(self.requests.len() as u32 / self.max_requests as u32),
            );
            self.backoff_until = Some(now + backoff_duration);
            false
        }
    }

    pub fn reset(&mut self) {
        self.requests.clear();
        self.backoff_until = None;
    }
}

/// Alert manager
#[cfg(feature = "server")]
pub struct AlertManager {
    telegram: Option<TelegramNotifier>,
    discord: Option<DiscordNotifier>,
    alert_history: Vec<Alert>,
    max_history: usize,
}

#[cfg(feature = "server")]
impl AlertManager {
    pub fn new() -> Self {
        Self {
            telegram: None,
            discord: None,
            alert_history: Vec::new(),
            max_history: 1000,
        }
    }

    pub fn with_telegram(mut self, bot_token: String, chat_id: String) -> Self {
        self.telegram = Some(TelegramNotifier::new(bot_token, chat_id));
        self
    }

    pub fn with_discord(mut self, webhook_url: String) -> Self {
        self.discord = Some(DiscordNotifier::new(webhook_url));
        self
    }

    /// Send alert to all configured channels
    pub async fn send_alert(&mut self, alert: Alert) -> Vec<Result<(), String>> {
        let mut results = Vec::new();

        // Send to Telegram
        if let Some(telegram) = &mut self.telegram {
            results.push(telegram.send_alert(&alert).await);
        }

        // Send to Discord
        if let Some(discord) = &mut self.discord {
            results.push(discord.send_alert(&alert).await);
        }

        // Store in history
        self.alert_history.push(alert);
        if self.alert_history.len() > self.max_history {
            self.alert_history.remove(0);
        }

        results
    }

    pub fn get_recent_alerts(&self, count: usize) -> &[Alert] {
        let start = self.alert_history.len().saturating_sub(count);
        &self.alert_history[start..]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(Duration::from_secs(1), 3);

        assert!(limiter.allow());
        assert!(limiter.allow());
        assert!(limiter.allow());
        assert!(!limiter.allow()); // Should be rate limited
    }

    #[test]
    fn test_alert_severity_ordering() {
        assert!(AlertSeverity::Critical > AlertSeverity::Error);
        assert!(AlertSeverity::Error > AlertSeverity::Warning);
        assert!(AlertSeverity::Warning > AlertSeverity::Info);
    }

    #[test]
    fn test_alert_creation() {
        let alert = Alert {
            alert_id: "test123".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            severity: AlertSeverity::Critical,
            title: "Node Failed".to_string(),
            message: "Node node1 has failed".to_string(),
            metadata: HashMap::new(),
        };

        assert_eq!(alert.severity, AlertSeverity::Critical);
        assert_eq!(alert.severity.emoji(), "🚨");
    }
}
