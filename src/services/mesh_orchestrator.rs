// src/services/mesh_orchestrator.rs
//! Mesh Orchestrator
//!
//! Manages distributed edge nodes and traffic routing

use crate::domain::models::{MeshNode, MeshNodeStatus, NodeCapacity, NodeHealth};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tonic::transport::{Channel, ClientTlsConfig};

use crate::rustray_client::rustray_proxyman::{
    AddInboundRequest, InboundHandlerConfig, handler_service_client::HandlerServiceClient,
};

/// Mesh orchestrator
pub struct MeshOrchestrator {
    nodes: HashMap<String, MeshNode>,
    routing_strategy: RoutingStrategy,
}

#[derive(Debug, Clone)]
pub enum RoutingStrategy {
    RoundRobin,
    LeastConnections,
    HealthBased,
    GeographicProximity,
}

impl MeshOrchestrator {
    pub fn new(strategy: RoutingStrategy) -> Self {
        Self {
            nodes: HashMap::new(),
            routing_strategy: strategy,
        }
    }

    /// Register a new node
    pub fn register_node(&mut self, node: MeshNode) {
        // Use node_id as key, or fallback to name if empty (though default uses uuid/placeholder)
        let key = if !node.node_id.is_empty() {
            node.node_id.clone()
        } else {
            node.name.clone()
        };
        self.nodes.insert(key, node);
    }

    /// Unregister a node
    pub fn unregister_node(&mut self, node_id: &str) -> Option<MeshNode> {
        self.nodes.remove(node_id)
    }

    /// Select best node for new user
    pub fn select_node(&self, user_region: Option<&str>) -> Option<&MeshNode> {
        let available_nodes: Vec<&MeshNode> = self
            .nodes
            .values()
            .filter(|n| n.status == MeshNodeStatus::Online && n.capacity.available_slots() > 0)
            .collect();

        if available_nodes.is_empty() {
            return None;
        }

        match self.routing_strategy {
            RoutingStrategy::RoundRobin => {
                // Simple: return first available
                available_nodes.first().copied()
            }
            RoutingStrategy::LeastConnections => available_nodes
                .iter()
                .min_by_key(|n| n.capacity.current_users)
                .copied(),
            RoutingStrategy::HealthBased => available_nodes
                .iter()
                .max_by(|a, b| {
                    a.health
                        .health_score()
                        .partial_cmp(&b.health.health_score())
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .copied(),
            RoutingStrategy::GeographicProximity => {
                if let Some(region) = user_region {
                    // Prefer nodes in same region
                    available_nodes
                        .iter()
                        .find(|n| n.region == region)
                        .or_else(|| available_nodes.first())
                        .copied()
                } else {
                    available_nodes.first().copied()
                }
            }
        }
    }

    /// Get mesh statistics
    pub fn get_mesh_stats(&self) -> MeshStats {
        let total_nodes = self.nodes.len();
        let online_nodes = self
            .nodes
            .values()
            .filter(|n| n.status == MeshNodeStatus::Online)
            .count();

        let total_capacity: u32 = self.nodes.values().map(|n| n.capacity.max_users).sum();
        let total_users: u32 = self.nodes.values().map(|n| n.capacity.current_users).sum();

        let avg_health = if !self.nodes.is_empty() {
            self.nodes
                .values()
                .map(|n| n.health.health_score())
                .sum::<f32>()
                / self.nodes.len() as f32
        } else {
            0.0
        };

        MeshStats {
            total_nodes,
            online_nodes,
            total_capacity,
            total_users,
            utilization_percent: if total_capacity > 0 {
                (total_users as f32 / total_capacity as f32) * 100.0
            } else {
                0.0
            },
            avg_health_score: avg_health,
        }
    }

    /// Rebalance users across nodes
    pub fn rebalance(&self) -> Vec<RebalanceAction> {
        let mut actions = Vec::new();

        // Find overloaded nodes (>80% utilization)
        let overloaded: Vec<&MeshNode> = self
            .nodes
            .values()
            .filter(|n| n.capacity.utilization_percent() > 80.0)
            .collect();

        // Find underutilized nodes (<50% utilization)
        let underutilized: Vec<&MeshNode> = self
            .nodes
            .values()
            .filter(|n| {
                n.status == MeshNodeStatus::Online && n.capacity.utilization_percent() < 50.0
            })
            .collect();

        for overloaded_node in overloaded {
            if let Some(target_node) = underutilized.first() {
                let users_to_move = (overloaded_node.capacity.current_users as f32 * 0.2) as u32;

                actions.push(RebalanceAction {
                    from_node: overloaded_node.node_id.clone(),
                    to_node: target_node.node_id.clone(),
                    user_count: users_to_move,
                    reason: "Load balancing".to_string(),
                });
            }
        }

        actions
    }

    /// List all nodes
    pub fn list_nodes(&self) -> Vec<MeshNode> {
        self.nodes.values().cloned().collect()
    }

    /// Get node by ID
    pub fn get_node(&self, node_id: &str) -> Option<&MeshNode> {
        self.nodes.get(node_id)
    }

    /// Update node health
    pub fn update_node_health(&mut self, node_id: &str, health: NodeHealth) {
        if let Some(node) = self.nodes.get_mut(node_id) {
            node.health = health.clone();

            // Auto-update status based on health
            if !health.is_healthy() {
                node.status = MeshNodeStatus::Degraded;
            } else if node.status == MeshNodeStatus::Degraded {
                node.status = MeshNodeStatus::Online;
            }
        }
    }

    /// Runs a Node Sync job to serialize and push inbounds to all "Active" nodes using gRPC-over-TLS with strict 2s timeouts.
    pub async fn run_node_sync_job(&self, inbounds: Vec<InboundHandlerConfig>) {
        let active_nodes: Vec<MeshNode> = self
            .nodes
            .values()
            .filter(|n| n.status == MeshNodeStatus::Online && !n.is_local)
            .cloned()
            .collect();

        for node in active_nodes {
            let inbounds_clone = inbounds.clone();
            tokio::spawn(async move {
                let url = format!("https://{}:{}", node.address, node.port);
                let tls = ClientTlsConfig::new();

                let channel_result = match Channel::from_shared(url) {
                    Ok(endpoint) => {
                        match endpoint.tls_config(tls) {
                            Ok(ep) => {
                                ep.timeout(Duration::from_secs(2))
                                    .connect()
                                    .await
                            }
                            Err(e) => Err(e),
                        }
                    }
                    Err(e) => {
                        log::error!("Invalid URL for node {}: {}", node.node_id, e);
                        return;
                    }
                };

                let channel = match channel_result {
                    Ok(c) => c,
                    Err(e) => {
                        log::error!("Failed to connect to active node {}: {}", node.node_id, e);
                        return;
                    }
                };

                let mut client = HandlerServiceClient::new(channel);

                for inbound in inbounds_clone {
                    let req = AddInboundRequest {
                        inbound: Some(inbound),
                    };
                    // Push config with strict 2s timeout per call
                    let res =
                        tokio::time::timeout(Duration::from_secs(2), client.add_inbound(req)).await;

                    if let Err(e) = res {
                        log::error!("Timeout pushing inbound to {}: {}", node.node_id, e);
                    } else if let Ok(Err(e)) = res {
                        log::error!("Error pushing inbound to {}: {}", node.node_id, e);
                    }
                }
            });
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshStats {
    pub total_nodes: usize,
    pub online_nodes: usize,
    pub total_capacity: u32,
    pub total_users: u32,
    pub utilization_percent: f32,
    pub avg_health_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebalanceAction {
    pub from_node: String,
    pub to_node: String,
    pub user_count: u32,
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::MeshNodeRole;

    fn create_test_node(id: &str, current_users: u32) -> MeshNode {
        MeshNode {
            node_id: id.to_string(),
            region: "us-east".to_string(),
            address: "127.0.0.1".to_string(),
            port: 443,
            capacity: NodeCapacity {
                max_users: 100,
                max_bandwidth_mbps: 1000,
                current_users,
                current_bandwidth_mbps: 200,
            },
            health: NodeHealth {
                cpu_percent: 50.0,
                memory_percent: 60.0,
                disk_percent: 40.0,
                latency_ms: 50.0,
                packet_loss_percent: 0.1,
            },
            status: MeshNodeStatus::Online,
            ..Default::default()
        }
    }

    #[test]
    fn test_node_selection_least_connections() {
        let mut orchestrator = MeshOrchestrator::new(RoutingStrategy::LeastConnections);

        orchestrator.register_node(create_test_node("node1", 50));
        orchestrator.register_node(create_test_node("node2", 30));
        orchestrator.register_node(create_test_node("node3", 70));

        let selected = orchestrator.select_node(None).unwrap();
        assert_eq!(selected.node_id, "node2"); // Least connections
    }

    #[test]
    fn test_mesh_stats() {
        let mut orchestrator = MeshOrchestrator::new(RoutingStrategy::RoundRobin);

        orchestrator.register_node(create_test_node("node1", 50));
        orchestrator.register_node(create_test_node("node2", 30));

        let stats = orchestrator.get_mesh_stats();
        assert_eq!(stats.total_nodes, 2);
        assert_eq!(stats.total_users, 80);
        assert_eq!(stats.total_capacity, 200);
    }

    #[test]
    fn test_rebalancing() {
        let mut orchestrator = MeshOrchestrator::new(RoutingStrategy::RoundRobin);

        orchestrator.register_node(create_test_node("node1", 85)); // Overloaded
        orchestrator.register_node(create_test_node("node2", 20)); // Underutilized

        let actions = orchestrator.rebalance();
        assert!(!actions.is_empty());
        assert_eq!(actions[0].from_node, "node1");
        assert_eq!(actions[0].to_node, "node2");
    }

    #[test]
    fn test_health_score() {
        let health = NodeHealth {
            cpu_percent: 20.0,
            memory_percent: 30.0,
            disk_percent: 40.0,
            latency_ms: 50.0,
            packet_loss_percent: 0.1,
        };

        assert!(health.is_healthy());
        assert!(health.health_score() > 0.7);
    }
}
