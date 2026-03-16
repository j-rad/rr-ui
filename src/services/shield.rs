use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BanInfo {
    pub ip: IpAddr,
    pub attempts: u8,
    pub last_attempt: DateTime<Utc>,
    pub banned_until: Option<DateTime<Utc>>,
}

pub struct ShieldService {
    state: Arc<RwLock<HashMap<IpAddr, BanInfo>>>,
}

impl ShieldService {
    pub fn new() -> Self {
        let state = Arc::new(RwLock::new(HashMap::new()));
        let state_clone = state.clone();

        // Spawn a background cleanup task?
        // Or leave it to external call? Instruction said "cleanup Logic".
        // A simple tokio spawn is good.
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(600)).await; // Every 10 mins
                ShieldService::cleanup_internal(&state_clone);
            }
        });

        Self { state }
    }

    pub fn check_ip(&self, ip: IpAddr) -> bool {
        let map = self.state.read().unwrap();
        if let Some(info) = map.get(&ip) {
            if let Some(banned_until) = info.banned_until {
                if Utc::now() < banned_until {
                    return false; // Banned
                }
            }
        }
        true
    }

    pub fn parse_log_line(&self, log_line: &str) {
        // e.g., "auth failed for 192.168.1.100" or "unauthorized access from 10.0.0.5"
        let lower = log_line.to_lowercase();
        if lower.contains("auth failed") || lower.contains("unauthorized") {
            // Very naive IP extraction for illustration
            let words: Vec<&str> = log_line.split_whitespace().collect();
            for word in words {
                if let Ok(ip) = word
                    .trim_matches(|c: char| !c.is_alphanumeric() && c != '.' && c != ':')
                    .parse::<IpAddr>()
                {
                    self.record_failure(ip);
                    break;
                }
            }
        }
    }

    pub fn record_failure(&self, ip: IpAddr) {
        let now = Utc::now();
        let mut map = self.state.write().unwrap();
        let mut entry = map.entry(ip).or_insert(BanInfo {
            ip,
            attempts: 0,
            last_attempt: now,
            banned_until: None,
        });

        if entry.banned_until.is_none()
            && now.signed_duration_since(entry.last_attempt) > Duration::minutes(10)
        {
            entry.attempts = 0;
        }

        entry.attempts += 1;
        entry.last_attempt = now;

        if entry.attempts >= 5 && entry.banned_until.is_none() {
            entry.banned_until = Some(now + Duration::hours(24));
            log::warn!(
                "Brute force detected from {}. Banning via nftables for 24 hours.",
                ip
            );

            // Execute nftables blocking logic
            let _ = std::process::Command::new("nft")
                .args([
                    "add",
                    "element",
                    "inet",
                    "filter",
                    "blackhole",
                    &format!("{{ {} }}", ip),
                ])
                .output();
        }
    }

    pub fn reset(&self, ip: IpAddr) {
        self.state.write().unwrap().remove(&ip);
        // Remove from nftables
        let _ = std::process::Command::new("nft")
            .args([
                "delete",
                "element",
                "inet",
                "filter",
                "blackhole",
                &format!("{{ {} }}", ip),
            ])
            .output();
    }

    pub fn list_banned(&self) -> Vec<BanInfo> {
        self.state
            .read()
            .unwrap()
            .values()
            .filter(|v| v.banned_until.map(|t| t > Utc::now()).unwrap_or(false))
            .cloned()
            .collect()
    }

    fn cleanup_internal(state: &Arc<RwLock<HashMap<IpAddr, BanInfo>>>) {
        let now = Utc::now();
        let mut map = state.write().unwrap();
        map.retain(|ip, v| {
            if let Some(banned_until) = v.banned_until {
                let keep = banned_until > now;
                if !keep {
                    // Unban via nftables
                    let _ = std::process::Command::new("nft")
                        .args([
                            "delete",
                            "element",
                            "inet",
                            "filter",
                            "blackhole",
                            &format!("{{ {} }}", ip),
                        ])
                        .output();
                }
                keep
            } else {
                now.signed_duration_since(v.last_attempt) < Duration::minutes(20)
            }
        });
    }
}
