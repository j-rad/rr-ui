// src/services/live_quota.rs
//! Live Quota System
//!
//! Real-time quota enforcement using SurrealDB Live Queries

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Quota violation alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaAlert {
    pub alert_id: String,
    pub user_id: String,
    pub alert_type: AlertType,
    pub timestamp: i64,
    pub details: AlertDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertType {
    QuotaExceeded,
    QuotaWarning,
    ExpiryWarning,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertDetails {
    pub current_usage_gb: f64,
    pub quota_gb: u64,
    pub percent_used: f32,
    pub days_until_expiry: Option<i64>,
}

/// Live quota tracker
#[cfg(feature = "server")]
pub struct LiveQuotaTracker {
    user_quotas: HashMap<String, UserQuotaState>,
    alert_threshold: f32, // Percentage (e.g., 0.9 for 90%)
}

#[cfg(feature = "server")]
#[derive(Debug, Clone)]
struct UserQuotaState {
    user_id: String,
    quota_gb: u64,
    used_gb: f64,
    expires_at: i64,
    last_check: i64,
}

#[cfg(feature = "server")]
impl LiveQuotaTracker {
    pub fn new(alert_threshold: f32) -> Self {
        Self {
            user_quotas: HashMap::new(),
            alert_threshold,
        }
    }

    /// Update user quota state from traffic data
    pub fn update_user_quota(
        &mut self,
        user_id: String,
        quota_gb: u64,
        upload_bytes: u64,
        download_bytes: u64,
        expires_at: i64,
    ) -> Option<QuotaAlert> {
        let used_gb = (upload_bytes + download_bytes) as f64 / 1_073_741_824.0;
        let percent_used = (used_gb / quota_gb as f64) as f32;
        let now = chrono::Utc::now().timestamp();

        let state = UserQuotaState {
            user_id: user_id.clone(),
            quota_gb,
            used_gb,
            expires_at,
            last_check: now,
        };

        self.user_quotas.insert(user_id.clone(), state);

        // Check for violations
        self.check_quota_violation(&user_id, percent_used, used_gb, quota_gb, expires_at)
    }

    fn check_quota_violation(
        &self,
        user_id: &str,
        percent_used: f32,
        used_gb: f64,
        quota_gb: u64,
        expires_at: i64,
    ) -> Option<QuotaAlert> {
        let now = chrono::Utc::now().timestamp();
        let days_until_expiry = (expires_at - now) / 86400;

        // Quota exceeded
        if percent_used >= 1.0 {
            return Some(QuotaAlert {
                alert_id: format!("alert_{}_{}", user_id, now),
                user_id: user_id.to_string(),
                alert_type: AlertType::QuotaExceeded,
                timestamp: now,
                details: AlertDetails {
                    current_usage_gb: used_gb,
                    quota_gb,
                    percent_used,
                    days_until_expiry: Some(days_until_expiry),
                },
            });
        }

        // Quota warning
        if percent_used >= self.alert_threshold {
            return Some(QuotaAlert {
                alert_id: format!("alert_{}_{}", user_id, now),
                user_id: user_id.to_string(),
                alert_type: AlertType::QuotaWarning,
                timestamp: now,
                details: AlertDetails {
                    current_usage_gb: used_gb,
                    quota_gb,
                    percent_used,
                    days_until_expiry: Some(days_until_expiry),
                },
            });
        }

        // Expiry warning (7 days)
        if days_until_expiry <= 7 && days_until_expiry > 0 {
            return Some(QuotaAlert {
                alert_id: format!("alert_{}_{}", user_id, now),
                user_id: user_id.to_string(),
                alert_type: AlertType::ExpiryWarning,
                timestamp: now,
                details: AlertDetails {
                    current_usage_gb: used_gb,
                    quota_gb,
                    percent_used,
                    days_until_expiry: Some(days_until_expiry),
                },
            });
        }

        // Expired
        if days_until_expiry <= 0 {
            return Some(QuotaAlert {
                alert_id: format!("alert_{}_{}", user_id, now),
                user_id: user_id.to_string(),
                alert_type: AlertType::Expired,
                timestamp: now,
                details: AlertDetails {
                    current_usage_gb: used_gb,
                    quota_gb,
                    percent_used,
                    days_until_expiry: Some(days_until_expiry),
                },
            });
        }

        None
    }

    /// Get aggregated stats for a reseller
    pub fn get_reseller_stats(&self, user_ids: &[String]) -> ResellerQuotaStats {
        let mut total_quota_gb = 0u64;
        let mut total_used_gb = 0.0f64;
        let mut active_users = 0usize;
        let mut exceeded_users = 0usize;

        for user_id in user_ids {
            if let Some(state) = self.user_quotas.get(user_id) {
                total_quota_gb += state.quota_gb;
                total_used_gb += state.used_gb;
                active_users += 1;

                if state.used_gb >= state.quota_gb as f64 {
                    exceeded_users += 1;
                }
            }
        }

        ResellerQuotaStats {
            total_quota_gb,
            total_used_gb,
            active_users,
            exceeded_users,
            percent_used: if total_quota_gb > 0 {
                (total_used_gb / total_quota_gb as f64) as f32
            } else {
                0.0
            },
        }
    }

    /// SurrealDB Live Query for real-time quota monitoring
    pub fn get_live_query_sql() -> &'static str {
        r#"
        LIVE SELECT * FROM user WHERE 
            (traffic.upload_bytes + traffic.download_bytes) / 1073741824 >= quota.total_gb * 0.9
            OR expires_at - time::now() <= 7d
        "#
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResellerQuotaStats {
    pub total_quota_gb: u64,
    pub total_used_gb: f64,
    pub active_users: usize,
    pub exceeded_users: usize,
    pub percent_used: f32,
}

/// Quota enforcement action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuotaAction {
    Throttle { user_id: String, speed_kbps: u32 },
    Block { user_id: String },
    Notify { user_id: String, message: String },
}

#[cfg(feature = "server")]
pub struct QuotaEnforcer;

#[cfg(feature = "server")]
impl QuotaEnforcer {
    /// Enforce quota based on alert
    pub fn enforce(alert: &QuotaAlert) -> Vec<QuotaAction> {
        let mut actions = Vec::new();

        match alert.alert_type {
            AlertType::QuotaExceeded => {
                // Block user
                actions.push(QuotaAction::Block {
                    user_id: alert.user_id.clone(),
                });
                actions.push(QuotaAction::Notify {
                    user_id: alert.user_id.clone(),
                    message: format!(
                        "Quota exceeded: {:.2}GB / {}GB used",
                        alert.details.current_usage_gb, alert.details.quota_gb
                    ),
                });
            }
            AlertType::QuotaWarning => {
                // Throttle to 50% speed
                actions.push(QuotaAction::Throttle {
                    user_id: alert.user_id.clone(),
                    speed_kbps: 5000,
                });
                actions.push(QuotaAction::Notify {
                    user_id: alert.user_id.clone(),
                    message: format!(
                        "Quota warning: {:.0}% used",
                        alert.details.percent_used * 100.0
                    ),
                });
            }
            AlertType::ExpiryWarning => {
                actions.push(QuotaAction::Notify {
                    user_id: alert.user_id.clone(),
                    message: format!(
                        "Account expires in {} days",
                        alert.details.days_until_expiry.unwrap_or(0)
                    ),
                });
            }
            AlertType::Expired => {
                actions.push(QuotaAction::Block {
                    user_id: alert.user_id.clone(),
                });
                actions.push(QuotaAction::Notify {
                    user_id: alert.user_id.clone(),
                    message: "Account expired".to_string(),
                });
            }
        }

        actions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quota_exceeded_alert() {
        let mut tracker = LiveQuotaTracker::new(0.9);

        let alert = tracker.update_user_quota(
            "user1".to_string(),
            100,
            60_000_000_000, // 60GB upload
            50_000_000_000, // 50GB download (total 110GB > 100GB)
            chrono::Utc::now().timestamp() + 86400 * 30,
        );

        assert!(alert.is_some());
        if let Some(a) = alert {
            assert!(matches!(a.alert_type, AlertType::QuotaExceeded));
        }
    }

    #[test]
    fn test_quota_warning_alert() {
        let mut tracker = LiveQuotaTracker::new(0.9);

        let alert = tracker.update_user_quota(
            "user1".to_string(),
            100,
            50 * 1_073_741_824, // 50GiB upload
            45 * 1_073_741_824, // 45GiB download (total 95GiB = 95%)
            chrono::Utc::now().timestamp() + 86400 * 30,
        );

        assert!(alert.is_some());
        if let Some(a) = alert {
            assert!(matches!(a.alert_type, AlertType::QuotaWarning));
        }
    }

    #[test]
    fn test_reseller_stats() {
        let mut tracker = LiveQuotaTracker::new(0.9);

        tracker.update_user_quota(
            "user1".to_string(),
            100,
            50_000_000_000,
            0,
            chrono::Utc::now().timestamp() + 86400 * 30,
        );

        tracker.update_user_quota(
            "user2".to_string(),
            200,
            100_000_000_000,
            0,
            chrono::Utc::now().timestamp() + 86400 * 30,
        );

        let stats = tracker.get_reseller_stats(&["user1".to_string(), "user2".to_string()]);
        assert_eq!(stats.active_users, 2);
        assert_eq!(stats.total_quota_gb, 300);
    }
}
