use crate::db::DbClient;
use crate::models::Inbound;
use crate::services::orchestrator::{CoreOrchestrator, Orchestrator};
use log::{error, info, warn};
use std::sync::Arc;
use std::time::Duration;

pub struct StateReconciler {
    db: DbClient,
    orchestrator: Arc<Orchestrator>,
    app_state: Arc<crate::server::AppState>,
    trigger_rx: tokio::sync::mpsc::Receiver<()>,
}

impl StateReconciler {
    pub fn new(
        db: DbClient,
        orchestrator: Arc<Orchestrator>,
        app_state: Arc<crate::server::AppState>,
        trigger_rx: tokio::sync::mpsc::Receiver<()>,
    ) -> Self {
        Self {
            db,
            orchestrator,
            app_state,
            trigger_rx,
        }
    }

    pub async fn run(mut self) {
        let mut interval = tokio::time::interval(Duration::from_secs(60)); // Check every minute

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    info!("Starting periodic user reconciliation...");
                }
                _ = self.trigger_rx.recv() => {
                    info!("Manual reconciliation triggered.");
                }
            }

            if let Err(e) = self.reconcile().await {
                error!("Reconciliation failed: {}", e);
            }
        }
    }

    pub async fn reconcile(&self) -> anyhow::Result<()> {
        use crate::api::rustray_control::ConfigEvent;
        use crate::rustray_config::RustRayConfigBuilder;

        // 1. Fetch all nodes from mesh
        let nodes = self
            .app_state
            .mesh_orchestrator
            .list_nodes()
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        // 2. Generate desired configuration from DB
        let desired_config = RustRayConfigBuilder::build(&self.db).await?;
        let desired_hash = desired_config.calculate_hash()?;
        let json_config = serde_json::to_string(&desired_config)?;

        for node in nodes {
            let node_id = node.node_id.clone();

            // 3. Check for drift
            let needs_update = match &node.config_hash {
                Some(h) => h != &desired_hash,
                None => true,
            };

            if needs_update {
                info!(
                    "Drift detected for node '{}'. desired: {}, actual: {:?}",
                    node_id, desired_hash, node.config_hash
                );

                // 4. Push update via gRPC if subscriber exists
                if let Some(sender) = self.app_state.node_streams.get(&node_id) {
                    let event = ConfigEvent {
                        json_delta: json_config.clone(), // Sending full JSON as "delta" for now
                        config_hash: desired_hash.clone(),
                    };

                    if let Err(e) = sender.try_send(Ok(event)) {
                        warn!("Failed to push config update to node '{}': {}", node_id, e);
                    } else {
                        info!("Config update pushed to node '{}'", node_id);
                    }
                } else {
                    warn!(
                        "No active gRPC stream for node '{}', skipping update",
                        node_id
                    );
                }
            }
        }

        info!("Reconciliation complete.");
        Ok(())
    }
}
