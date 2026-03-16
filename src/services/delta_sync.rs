// src/services/delta_sync.rs
//! Delta Sync Engine
//!
//! Implements semantic diff calculation and delta push to edge nodes

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration delta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigDelta {
    pub delta_id: String,
    pub timestamp: i64,
    pub node_id: String,
    pub changes: Vec<Change>,
    pub checksum: String,
}

/// Individual change in a delta
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Change {
    UserAdded {
        user_id: String,
        config: UserConfig,
    },
    UserRemoved {
        user_id: String,
    },
    UserModified {
        user_id: String,
        field: String,
        old_value: serde_json::Value,
        new_value: serde_json::Value,
    },
    InboundAdded {
        inbound_id: String,
        config: InboundConfig,
    },
    InboundRemoved {
        inbound_id: String,
    },
    InboundModified {
        inbound_id: String,
        field: String,
        old_value: serde_json::Value,
        new_value: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub email: String,
    pub uuid: String,
    pub quota_gb: u64,
    pub expires_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundConfig {
    pub protocol: String,
    pub port: u16,
    pub settings: HashMap<String, serde_json::Value>,
}

/// Delta calculator
pub struct DeltaCalculator;

impl DeltaCalculator {
    /// Calculate semantic diff between two configs
    pub fn calculate(
        old_config: &serde_json::Value,
        new_config: &serde_json::Value,
    ) -> Vec<Change> {
        let mut changes = Vec::new();

        // Compare inbounds
        if let (Some(old_inbounds), Some(new_inbounds)) = (
            old_config.get("inbounds").and_then(|v| v.as_array()),
            new_config.get("inbounds").and_then(|v| v.as_array()),
        ) {
            changes.extend(Self::diff_inbounds(old_inbounds, new_inbounds));
        }

        // Compare users (clients)
        if let (Some(old_users), Some(new_users)) = (
            old_config.get("users").and_then(|v| v.as_array()),
            new_config.get("users").and_then(|v| v.as_array()),
        ) {
            changes.extend(Self::diff_users(old_users, new_users));
        }

        changes
    }

    fn diff_inbounds(old: &[serde_json::Value], new: &[serde_json::Value]) -> Vec<Change> {
        let mut changes = Vec::new();

        let old_map: HashMap<String, &serde_json::Value> = old
            .iter()
            .filter_map(|v| {
                v.get("id")
                    .and_then(|id| id.as_str())
                    .map(|id| (id.to_string(), v))
            })
            .collect();

        let new_map: HashMap<String, &serde_json::Value> = new
            .iter()
            .filter_map(|v| {
                v.get("id")
                    .and_then(|id| id.as_str())
                    .map(|id| (id.to_string(), v))
            })
            .collect();

        // Find added inbounds
        for (id, config) in &new_map {
            if !old_map.contains_key(id) {
                if let Ok(inbound_config) = serde_json::from_value((*config).clone()) {
                    changes.push(Change::InboundAdded {
                        inbound_id: id.clone(),
                        config: inbound_config,
                    });
                }
            }
        }

        // Find removed inbounds
        for id in old_map.keys() {
            if !new_map.contains_key(id) {
                changes.push(Change::InboundRemoved {
                    inbound_id: id.clone(),
                });
            }
        }

        // Find modified inbounds
        for (id, new_val) in &new_map {
            if let Some(old_val) = old_map.get(id) {
                if old_val != new_val {
                    // Simplified: just mark as modified
                    // In production, would do field-level diff
                    changes.push(Change::InboundModified {
                        inbound_id: id.clone(),
                        field: "config".to_string(),
                        old_value: (*old_val).clone(),
                        new_value: (*new_val).clone(),
                    });
                }
            }
        }

        changes
    }

    fn diff_users(old: &[serde_json::Value], new: &[serde_json::Value]) -> Vec<Change> {
        let mut changes = Vec::new();

        let old_map: HashMap<String, &serde_json::Value> = old
            .iter()
            .filter_map(|v| {
                v.get("id")
                    .and_then(|id| id.as_str())
                    .map(|id| (id.to_string(), v))
            })
            .collect();

        let new_map: HashMap<String, &serde_json::Value> = new
            .iter()
            .filter_map(|v| {
                v.get("id")
                    .and_then(|id| id.as_str())
                    .map(|id| (id.to_string(), v))
            })
            .collect();

        // Find added users
        for (id, config) in &new_map {
            if !old_map.contains_key(id) {
                if let Ok(user_config) = serde_json::from_value((*config).clone()) {
                    changes.push(Change::UserAdded {
                        user_id: id.clone(),
                        config: user_config,
                    });
                }
            }
        }

        // Find removed users
        for id in old_map.keys() {
            if !new_map.contains_key(id) {
                changes.push(Change::UserRemoved {
                    user_id: id.clone(),
                });
            }
        }

        // Find modified users
        for (id, new_val) in &new_map {
            if let Some(old_val) = old_map.get(id) {
                if old_val != new_val {
                    changes.push(Change::UserModified {
                        user_id: id.clone(),
                        field: "config".to_string(),
                        old_value: (*old_val).clone(),
                        new_value: (*new_val).clone(),
                    });
                }
            }
        }

        changes
    }

    /// Calculate checksum for delta
    pub fn calculate_checksum(changes: &[Change]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let serialized = serde_json::to_string(changes).unwrap_or_default();
        let mut hasher = DefaultHasher::new();
        serialized.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

/// Delta sync manager
#[cfg(feature = "server")]
pub struct DeltaSyncManager {
    node_configs: HashMap<String, serde_json::Value>,
}

#[cfg(feature = "server")]
impl DeltaSyncManager {
    pub fn new() -> Self {
        Self {
            node_configs: HashMap::new(),
        }
    }

    /// Generate delta for a node
    pub fn generate_delta(&mut self, node_id: &str, new_config: serde_json::Value) -> ConfigDelta {
        let old_config = self
            .node_configs
            .get(node_id)
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));

        let changes = DeltaCalculator::calculate(&old_config, &new_config);
        let checksum = DeltaCalculator::calculate_checksum(&changes);

        // Update stored config
        self.node_configs.insert(node_id.to_string(), new_config);

        ConfigDelta {
            delta_id: format!("delta_{}_{}", node_id, chrono::Utc::now().timestamp()),
            timestamp: chrono::Utc::now().timestamp(),
            node_id: node_id.to_string(),
            changes,
            checksum,
        }
    }

    /// Apply delta to a config
    pub fn apply_delta(config: &mut serde_json::Value, delta: &ConfigDelta) -> Result<(), String> {
        for change in &delta.changes {
            match change {
                Change::UserAdded {
                    user_id,
                    config: user_config,
                } => {
                    if let Some(users) = config.get_mut("users").and_then(|v| v.as_array_mut()) {
                        users.push(serde_json::to_value(user_config).unwrap());
                    }
                }
                Change::UserRemoved { user_id } => {
                    if let Some(users) = config.get_mut("users").and_then(|v| v.as_array_mut()) {
                        users.retain(|u| u.get("id").and_then(|id| id.as_str()) != Some(user_id));
                    }
                }
                Change::UserModified {
                    user_id,
                    field,
                    new_value,
                    ..
                } => {
                    if let Some(users) = config.get_mut("users").and_then(|v| v.as_array_mut()) {
                        for user in users {
                            if user.get("id").and_then(|id| id.as_str()) == Some(user_id) {
                                if let Some(obj) = user.as_object_mut() {
                                    obj.insert(field.clone(), new_value.clone());
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_calculation() {
        let old = serde_json::json!({
            "users": [
                {"id": "user1", "email": "test@example.com", "uuid": "uuid1", "quota_gb": 10, "expires_at": 1000}
            ]
        });

        let new = serde_json::json!({
            "users": [
                {"id": "user1", "email": "test@example.com", "uuid": "uuid1", "quota_gb": 10, "expires_at": 1000},
                {"id": "user2", "email": "new@example.com", "uuid": "uuid2", "quota_gb": 20, "expires_at": 2000}
            ]
        });

        let changes = DeltaCalculator::calculate(&old, &new);
        assert!(!changes.is_empty());
    }

    #[test]
    fn test_checksum_generation() {
        let changes = vec![Change::UserRemoved {
            user_id: "test".to_string(),
        }];
        let checksum = DeltaCalculator::calculate_checksum(&changes);
        assert!(!checksum.is_empty());
    }
}
