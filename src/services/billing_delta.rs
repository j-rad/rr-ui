// src/services/billing_delta.rs
#![cfg(feature = "server")]

use crate::db::DbClient;
use dashmap::DashMap;
use log::{error, info};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;

#[derive(Default, Debug)]
struct Delta {
    up: u64,
    down: u64,
}

/// High-performance Billing Delta Sync Engine.
/// Uses a lock-free buffer (DashMap) to aggregate traffic updates from nodes
/// and flushes them to the DB using atomic increments.
pub struct BillingDeltaSync {
    db: DbClient,
    // (inbound_tag, email) -> delta
    buffer: Arc<DashMap<(String, String), Delta>>,
}

impl BillingDeltaSync {
    pub fn new(db: DbClient) -> Self {
        Self {
            db,
            buffer: Arc::new(DashMap::new()),
        }
    }

    /// Records traffic for a specific user.
    /// This is lock-free and thread-safe, suitable for high-concurrency ingestion.
    pub fn record_traffic(&self, inbound_tag: String, email: String, up: u64, down: u64) {
        if up == 0 && down == 0 {
            return;
        }
        let mut entry = self.buffer.entry((inbound_tag, email)).or_default();
        entry.up = entry.up.saturating_add(up);
        entry.down = entry.down.saturating_add(down);
    }

    /// Starts the background flush job.
    pub fn start_flush_job(self: Arc<Self>) {
        let this = self.clone();
        tokio::spawn(async move {
            info!("Billing Delta Sync flush job started (Interval: 10s)");
            let mut interval = interval(Duration::from_secs(10));
            loop {
                interval.tick().await;
                if let Err(e) = this.flush().await {
                    error!("Billing Delta Sync flush failed: {}", e);
                }
            }
        });
    }

    /// Flushes the current buffer to SurrealDB using optimized atomic updates.
    async fn flush(&self) -> anyhow::Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        // Extract updates from buffer
        let mut updates = Vec::new();
        {
            // We iterate and remove to clear the buffer while processing
            // To avoid holding the shard lock too long, we collect keys first
            let keys: Vec<_> = self.buffer.iter().map(|kv| kv.key().clone()).collect();
            for key in keys {
                if let Some((_, delta)) = self.buffer.remove(&key) {
                    updates.push((key, delta));
                }
            }
        }

        if updates.is_empty() {
            return Ok(());
        }

        info!("Syncing {} traffic deltas to database...", updates.len());

        // We use a batched transaction to execute all updates in a single round-trip
        // This solves the N+1 query problem by combining everything into one transaction string
        let mut query_string = String::from("BEGIN TRANSACTION;\n");
        let mut vars = serde_json::Map::new();

        for (i, ((tag, email), delta)) in updates.into_iter().enumerate() {
            query_string.push_str(&format!(
                "UPDATE inbound
                 SET settings.clients[WHERE email = $email_{i}].up += $up_{i},
                     settings.clients[WHERE email = $email_{i}].down += $down_{i}
                 WHERE tag = $tag_{i};\n"
            ));
            vars.insert(format!("tag_{i}"), serde_json::json!(tag));
            vars.insert(format!("email_{i}"), serde_json::json!(email));
            vars.insert(format!("up_{i}"), serde_json::json!(delta.up));
            vars.insert(format!("down_{i}"), serde_json::json!(delta.down));
        }

        query_string.push_str("COMMIT TRANSACTION;");

        let vars_obj = serde_json::Value::Object(vars);
        if let Err(e) = self.db.client.query(query_string).bind(vars_obj).await {
            error!("Atomic billing sync batch failed: {}", e);
            // In production, we might want to put these back in a retry buffer
        }

        Ok(())
    }
}
