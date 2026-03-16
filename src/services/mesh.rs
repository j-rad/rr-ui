// src/services/mesh.rs
//! Mesh Orchestration Service
//!
//! Manages multi-node cluster coordination, node discovery, and heartbeat synchronization.

use crate::db::DbClient;
use crate::models::{ClusterStats, MeshNode};
use log::{error, info, warn};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Mesh orchestration service for multi-node coordination
#[derive(Clone)]
pub struct MeshOrchestrator {
    db: Arc<RwLock<DbClient>>,
    local_node: Arc<RwLock<Option<MeshNode>>>,
    heartbeat_interval: Duration,
}

impl MeshOrchestrator {
    /// Create a new mesh orchestrator
    pub fn new(db: Arc<RwLock<DbClient>>) -> Self {
        Self {
            db,
            local_node: Arc::new(RwLock::new(None)),
            heartbeat_interval: Duration::from_secs(30),
        }
    }

    /// Initialize this node in the mesh
    pub async fn init_local_node(&self, name: &str, port: u16) -> Result<(), String> {
        let node = MeshNode::local(name, port);

        // Store in database
        let db = self.db.read().await;
        db.client
            .create::<Option<MeshNode>>("mesh_node")
            .content(node.clone())
            .await
            .map_err(|e| format!("Failed to register local node: {}", e))?;

        // Cache locally
        *self.local_node.write().await = Some(node);

        info!("Mesh: Local node '{}' registered on port {}", name, port);
        Ok(())
    }

    /// Get the local node
    pub async fn get_local_node(&self) -> Option<MeshNode> {
        self.local_node.read().await.clone()
    }

    /// Register a remote node
    pub async fn register_node(&self, node: MeshNode) -> Result<MeshNode, String> {
        let db = self.db.read().await;

        let created: Option<MeshNode> = db
            .client
            .create("mesh_node")
            .content(node.clone())
            .await
            .map_err(|e| format!("Failed to register node: {}", e))?;

        info!("Mesh: Registered node '{}' at {}", node.name, node.address);
        created.ok_or_else(|| "Failed to create node".to_string())
    }

    /// Get all registered nodes
    pub async fn list_nodes(&self) -> Result<Vec<MeshNode>, String> {
        let db = self.db.read().await;

        let nodes: Vec<MeshNode> = db
            .client
            .select("mesh_node")
            .await
            .map_err(|e| format!("Failed to list nodes: {}", e))?;

        Ok(nodes)
    }

    /// Get online nodes only
    pub async fn list_online_nodes(&self) -> Result<Vec<MeshNode>, String> {
        let nodes = self.list_nodes().await?;
        Ok(nodes.into_iter().filter(|n| !n.is_stale()).collect())
    }

    /// Send heartbeat for local node
    pub async fn heartbeat(&self) -> Result<(), String> {
        let mut local = self.local_node.write().await;
        if let Some(node) = local.as_mut() {
            node.heartbeat();

            // Update in database
            if let Some(ref id) = node.id {
                let db = self.db.read().await;
                let query = format!(
                    "UPDATE {} SET last_heartbeat = {}, status = 'online'",
                    id, node.last_heartbeat
                );
                db.client.query(&query).await.map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }

    /// Check health of all nodes and mark stale ones as offline
    pub async fn check_node_health(&self) -> Result<(), String> {
        let nodes = self.list_nodes().await?;
        let db = self.db.read().await;

        for node in nodes {
            if node.is_stale() && !node.is_local {
                if let Some(ref id) = node.id {
                    let query = format!("UPDATE {} SET status = 'offline'", id);
                    if let Err(e) = db.client.query(&query).await {
                        warn!("Failed to mark node {} as offline: {}", node.name, e);
                    } else {
                        info!(
                            "Mesh: Node '{}' marked as offline (stale heartbeat)",
                            node.name
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Remove a node from the mesh
    pub async fn remove_node(&self, node_name: &str) -> Result<(), String> {
        let db = self.db.read().await;

        let query = format!("DELETE mesh_node WHERE name = '{}'", node_name);
        db.client.query(&query).await.map_err(|e| e.to_string())?;

        info!("Mesh: Node '{}' removed from mesh", node_name);
        Ok(())
    }

    /// Start the background heartbeat loop
    pub fn start_heartbeat_loop(self: Arc<Self>) {
        let orchestrator = self.clone();
        let interval = self.heartbeat_interval;

        tokio::spawn(async move {
            loop {
                if let Err(e) = orchestrator.heartbeat().await {
                    error!("Mesh heartbeat failed: {}", e);
                }

                if let Err(e) = orchestrator.check_node_health().await {
                    error!("Mesh health check failed: {}", e);
                }

                tokio::time::sleep(interval).await;
            }
        });
    }

    /// Get cluster statistics
    pub async fn get_cluster_stats(&self) -> Result<ClusterStats, String> {
        let nodes = self.list_nodes().await?;
        let online_count = nodes.iter().filter(|n| !n.is_stale()).count();
        let total_clients: u32 = nodes.iter().map(|n| n.client_count).sum();

        Ok(ClusterStats {
            total_nodes: nodes.len(),
            online_nodes: online_count,
            offline_nodes: nodes.len() - online_count,
            total_clients,
        })
    }
}

/// Shared mesh orchestrator type
pub type SharedMeshOrchestrator = Arc<MeshOrchestrator>;
