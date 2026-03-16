// src/services/bulk_worker.rs
//! Bulk Operations Worker
//!
//! Handles batch processing of client operations with progress tracking

#[cfg(feature = "server")]
use crate::db::DbClient;
use crate::domain::bulk_operations::*;
use anyhow::Result;
use std::sync::Arc;
use std::time::Instant;

/// Batch size for processing clients
const BATCH_SIZE: usize = 50;

/// Bulk operations worker
#[cfg(feature = "server")]
pub struct BulkWorker {
    db: Arc<DbClient>,
}

#[cfg(feature = "server")]
impl BulkWorker {
    pub fn new(db: Arc<DbClient>) -> Self {
        Self { db }
    }

    /// Execute a bulk operation
    ///
    /// Processes clients in batches of 50 for efficiency
    pub async fn execute(&self, request: BulkOperationRequest) -> Result<BulkOperationResult> {
        let start = Instant::now();
        let operation_id = format!("bulk_{}", start.elapsed().as_nanos());
        let mut progress = BulkOperationProgress::new(request.client_ids.len());

        // Process in batches
        for chunk in request.client_ids.chunks(BATCH_SIZE) {
            let chunk_ids: Vec<&str> = chunk.iter().map(|s| s.as_str()).collect();

            match self.execute_batch(&chunk_ids, &request.operation).await {
                Ok(batch_result) => {
                    for (id, result) in batch_result {
                        match result {
                            Ok(_) => progress.record_success(),
                            Err(e) => progress.record_failure(id, e),
                        }
                    }
                }
                Err(e) => {
                    // Batch-level error - mark all as failed
                    for id in chunk {
                        progress.record_failure(id.clone(), e.to_string());
                    }
                }
            }
        }

        progress.mark_complete();

        Ok(BulkOperationResult {
            operation_id,
            progress,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    /// Execute operation on a batch of clients
    async fn execute_batch(
        &self,
        client_ids: &[&str],
        operation: &BulkOperation,
    ) -> Result<Vec<(String, Result<(), String>)>> {
        let mut results = Vec::with_capacity(client_ids.len());
        // Clone for async use
        let client_ids_owned: Vec<String> = client_ids.iter().map(|s| s.to_string()).collect();

        match operation {
            // All operations that update specific fields using "WHERE id IN $ids"
            BulkOperation::Enable
            | BulkOperation::Disable
            | BulkOperation::ResetTraffic
            | BulkOperation::Delete => {
                let query = match operation {
                    BulkOperation::Enable => "UPDATE client SET enable = true WHERE id IN $ids",
                    BulkOperation::Disable => "UPDATE client SET enable = false WHERE id IN $ids",
                    BulkOperation::ResetTraffic => {
                        "UPDATE client SET up = 0, down = 0 WHERE id IN $ids"
                    }
                    BulkOperation::Delete => "DELETE client WHERE id IN $ids",
                    _ => unreachable!(),
                };

                match self
                    .db
                    .client
                    .query(query)
                    .bind(("ids", client_ids_owned.clone()))
                    .await
                {
                    Ok(_) => {
                        for id in client_ids {
                            results.push((id.to_string(), Ok(())));
                        }
                    }
                    Err(e) => {
                        for id in client_ids {
                            results.push((id.to_string(), Err(e.to_string())));
                        }
                    }
                }
            }

            BulkOperation::ExtendExpiry { days } => {
                let query = format!(
                    "UPDATE client SET expiry_time = expiry_time + {}d WHERE id IN $ids",
                    days
                );
                match self
                    .db
                    .client
                    .query(query)
                    .bind(("ids", client_ids_owned.clone()))
                    .await
                {
                    Ok(_) => {
                        for id in client_ids {
                            results.push((id.to_string(), Ok(())));
                        }
                    }
                    Err(e) => {
                        for id in client_ids {
                            results.push((id.to_string(), Err(e.to_string())));
                        }
                    }
                }
            }

            BulkOperation::SetExpiry { timestamp } => {
                let query = "UPDATE client SET expiry_time = $ts WHERE id IN $ids";
                match self
                    .db
                    .client
                    .query(query)
                    .bind(("ts", *timestamp))
                    .bind(("ids", client_ids_owned.clone()))
                    .await
                {
                    Ok(_) => {
                        for id in client_ids {
                            results.push((id.to_string(), Ok(())));
                        }
                    }
                    Err(e) => {
                        for id in client_ids {
                            results.push((id.to_string(), Err(e.to_string())));
                        }
                    }
                }
            }

            BulkOperation::SetQuota { total_gb } => {
                let query = "UPDATE client SET total_gb = $quota WHERE id IN $ids";
                match self
                    .db
                    .client
                    .query(query)
                    .bind(("quota", *total_gb))
                    .bind(("ids", client_ids_owned.clone()))
                    .await
                {
                    Ok(_) => {
                        for id in client_ids {
                            results.push((id.to_string(), Ok(())));
                        }
                    }
                    Err(e) => {
                        for id in client_ids {
                            results.push((id.to_string(), Err(e.to_string())));
                        }
                    }
                }
            }

            BulkOperation::AddQuota { additional_gb } => {
                let query = format!(
                    "UPDATE client SET total_gb = total_gb + {} WHERE id IN $ids",
                    additional_gb
                );
                match self
                    .db
                    .client
                    .query(query)
                    .bind(("ids", client_ids_owned.clone()))
                    .await
                {
                    Ok(_) => {
                        for id in client_ids {
                            results.push((id.to_string(), Ok(())));
                        }
                    }
                    Err(e) => {
                        for id in client_ids {
                            results.push((id.to_string(), Err(e.to_string())));
                        }
                    }
                }
            }

            BulkOperation::SetSpeedLimit {
                download_mbps,
                upload_mbps,
            } => {
                let mut updates = Vec::new();
                if let Some(dl) = download_mbps {
                    updates.push(format!("limit_down = {}", dl));
                }
                if let Some(ul) = upload_mbps {
                    updates.push(format!("limit_up = {}", ul));
                }

                if updates.is_empty() {
                    for id in client_ids {
                        results.push((id.to_string(), Ok(())));
                    }
                } else {
                    let query =
                        format!("UPDATE client SET {} WHERE id IN $ids", updates.join(", "));
                    match self
                        .db
                        .client
                        .query(query)
                        .bind(("ids", client_ids_owned.clone()))
                        .await
                    {
                        Ok(_) => {
                            for id in client_ids {
                                results.push((id.to_string(), Ok(())));
                            }
                        }
                        Err(e) => {
                            for id in client_ids {
                                results.push((id.to_string(), Err(e.to_string())));
                            }
                        }
                    }
                }
            }

            BulkOperation::MoveToInbound { inbound_id } => {
                let query = "UPDATE client SET inbound_id = $ib_id WHERE id IN $ids";
                match self
                    .db
                    .client
                    .query(query)
                    .bind(("ib_id", inbound_id.to_string())) // to_string() makes it owned
                    .bind(("ids", client_ids_owned.clone()))
                    .await
                {
                    Ok(_) => {
                        for id in client_ids {
                            results.push((id.to_string(), Ok(())));
                        }
                    }
                    Err(e) => {
                        for id in client_ids {
                            results.push((id.to_string(), Err(e.to_string())));
                        }
                    }
                }
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_size() {
        assert_eq!(BATCH_SIZE, 50);
    }
}
