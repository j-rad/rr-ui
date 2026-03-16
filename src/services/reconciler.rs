use crate::db::DbClient;
use crate::models::Inbound;
use crate::services::orchestrator::{CoreOrchestrator, Orchestrator};
use log::{error, info, warn};
use std::sync::Arc;
use std::time::Duration;

pub struct StateReconciler {
    db: DbClient,
    orchestrator: Arc<Orchestrator>,
}

impl StateReconciler {
    pub fn new(db: DbClient, orchestrator: Arc<Orchestrator>) -> Self {
        Self { db, orchestrator }
    }

    pub async fn run(self) {
        let mut interval = tokio::time::interval(Duration::from_secs(60)); // Check every minute

        loop {
            interval.tick().await;
            info!("Starting periodic user reconciliation...");

            if let Err(e) = self.reconcile().await {
                error!("Reconciliation failed: {}", e);
            }
        }
    }

    pub async fn reconcile(&self) -> anyhow::Result<()> {
        // 1. Fetch all inbounds from DB
        #[cfg(feature = "server")]
        {
            let sql = "SELECT * FROM inbound";
            let mut result = self.db.client.query(sql).await?;
            let inbounds: Vec<Inbound> = result.take(0)?;

            for inbound in inbounds {
                if !inbound.enable {
                    continue;
                }

                if let Some(clients) = inbound.settings.clients() {
                    for client in clients {
                        let uuid = client.id.as_deref().unwrap_or_default();
                        let email = client.email.as_deref().unwrap_or_default();

                        // Only sync enabled clients
                        if !client.enable {
                            continue;
                        }

                        // If no UUID/Email, skip (can't sync invalid user)
                        if uuid.is_empty() {
                            continue;
                        }

                        // Attempt sync
                        // We map client.level to u32, default 0
                        // Inbound tag comes from inbound struct
                        match self
                            .orchestrator
                            .sync_user_live(uuid, email, &inbound.tag, 0)
                            .await
                        {
                            Ok(_) => {
                                // Success - strictly speaking RustRay AddUser throws error if exists,
                                // but we might want to ignore "Generic error: User already exists"
                                // Since we don't strictly parsing the error string here, we log debug/warn.
                            }
                            Err(e) => {
                                // "User already exists" is a common error here if we naively add.
                                // Real implementation should check existence or handle specific error type.
                                // For now we log warning.
                                warn!(
                                    "Failed to sync user {} to inbound {}: {}",
                                    email, inbound.tag, e
                                );
                            }
                        }
                    }
                }
            }
        }

        info!("Reconciliation complete.");
        Ok(())
    }
}
