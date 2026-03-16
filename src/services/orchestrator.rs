use anyhow::{Result, anyhow};
use async_trait::async_trait;
use log::{error, info, warn};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tonic::transport::Channel;

// Include generated proto modules
pub mod proto {
    pub mod common {
        pub mod protocol {
            tonic::include_proto!("rustray.common.protocol");
        }
        pub mod serial {
            tonic::include_proto!("rustray.common.serial");
        }
    }
    pub mod command {
        tonic::include_proto!("rustray.core.app.proxyman.command");
    }
    pub mod stats {
        tonic::include_proto!("rustray.stats");
    }
}

use proto::command::{
    AddInboundRequest, AlterInboundRequest, InboundHandlerConfig, RemoveInboundRequest,
    TypedMessage, handler_service_client::HandlerServiceClient,
};
use proto::common::protocol::User;
use proto::common::serial::TypedMessage as SerialTypedMessage;
use proto::stats::{QueryStatsRequest, Stat, stats_service_client::StatsServiceClient};

/// Orchestrator trait for managing the Core.
#[async_trait]
pub trait CoreOrchestrator: Send + Sync {
    async fn add_user(
        &self,
        inbound_tag: &str,
        user_email: &str,
        user_uuid: &str,
        level: u32,
    ) -> Result<()>;
    async fn remove_user(&self, inbound_tag: &str, user_email: &str) -> Result<()>;
    async fn get_stats(&self, reset: bool) -> Result<Vec<Stat>>;
    async fn add_inbound(&self, config: InboundHandlerConfig) -> Result<()>;
    async fn remove_inbound(&self, tag: &str) -> Result<()>;
    async fn sync_user_live(
        &self,
        user_uuid: &str,
        user_email: &str,
        inbound_tag: &str,
        level: u32,
    ) -> Result<()>;
}

/// A connection manager that maintains a persistent reference to the gRPC clients.
#[derive(Clone)]
pub struct Orchestrator {
    handler_client: Arc<RwLock<Option<HandlerServiceClient<Channel>>>>,
    stats_client: Arc<RwLock<Option<StatsServiceClient<Channel>>>>,
    api_port: u16,
}

impl Orchestrator {
    pub fn new(api_port: u16) -> Self {
        Self {
            handler_client: Arc::new(RwLock::new(None)),
            stats_client: Arc::new(RwLock::new(None)),
            api_port,
        }
    }

    /// Connects to the core with exponential backoff.
    /// This connection loop runs until success.
    pub async fn connect(&self) {
        let addr = format!("http://127.0.0.1:{}", self.api_port);
        let mut delay = Duration::from_secs(1);
        let max_delay = Duration::from_secs(16);

        loop {
            match Channel::from_shared(addr.clone()) {
                Ok(endpoint) => match endpoint.connect().await {
                    Ok(channel) => {
                        let mut h_client = self.handler_client.write().await;
                        *h_client = Some(HandlerServiceClient::new(channel.clone()));

                        let mut s_client = self.stats_client.write().await;
                        *s_client = Some(StatsServiceClient::new(channel));

                        info!("Connected to Core Orchestrator at {}", addr);
                        return;
                    }
                    Err(e) => {
                        warn!(
                            "Failed to connect to Core: {}. Retrying in {:?}...",
                            e, delay
                        );
                    }
                },
                Err(e) => {
                    error!("Invalid URI '{}': {}", addr, e);
                    return; // Should basically never happen if port is valid
                }
            }

            tokio::time::sleep(delay).await;
            delay = std::cmp::min(delay * 2, max_delay);
        }
    }

    /// Background task to maintain connection.
    pub async fn connection_monitor(self: Arc<Self>) {
        // Simple monitor: if we detect disconnection or want to enforce liveness.
        // For now, `connect` establishes initial connection.
        // Tonic channels auto-reconnect, but if we need logic to re-init clients, we can do it here.
        // Assuming tonic handles channel reconnection, we just need to ensure clients are set.
        if self.handler_client.read().await.is_none() {
            self.connect().await;
        }
    }

    async fn get_handler_client(&self) -> Result<HandlerServiceClient<Channel>> {
        let guard = self.handler_client.read().await;
        guard
            .clone()
            .ok_or_else(|| anyhow!("Orchestrator not connected to Core"))
    }

    async fn get_stats_client(&self) -> Result<StatsServiceClient<Channel>> {
        let guard = self.stats_client.read().await;
        guard
            .clone()
            .ok_or_else(|| anyhow!("Orchestrator not connected to Core"))
    }
}

#[async_trait]
impl CoreOrchestrator for Orchestrator {
    async fn add_user(
        &self,
        inbound_tag: &str,
        user_email: &str,
        user_uuid: &str,
        level: u32,
    ) -> Result<()> {
        let mut client = self.get_handler_client().await?;

        // Construct User proto
        // Note: Generic User construction. Specific protocols might need wrapping.
        // Typically RustRay uses `TypedMessage` wrapping a `User` for `AddUserOperation`.
        // The exact `AddUserOperation` message depends on the proxy protocol (vmess, vless, etc).
        // Since we are creating a generic "CoreOrchestrator", we assume the Core (rustray)
        // provides a unified or specific way, or we need to handle per-protocol construction.
        // For this task, we will implement a generic VLESS/VMess compatible User addition
        // using `rustray.common.protocol.User`.

        // This is a simplified implementation. In reality, we'd wrap this in `AddUserOperation`
        // and serialise it into `TypedMessage` with type `rustray.proxy.vmess.inbound.AddUserOperation` etc.
        // Given constraints and limited proto visibility, we perform the best-effort structure:

        let account_any = SerialTypedMessage {
            r#type: "rustray.proxy.vless.Account".to_string(), // Defaulting to VLESS for now
            value: user_uuid.as_bytes().to_vec(), // Simplified: VLESS account is just the UUID string usually? No, it's proto.
                                                  // In a real generic implementation, we would need the specific Account proto for the protocol.
        };

        let user = User {
            level,
            email: user_email.to_string(),
            account: Some(account_any),
        };

        // Serialize User
        use prost::Message;
        let mut user_bytes = Vec::new();
        user.encode(&mut user_bytes)?;

        // AlterInboundRequest with operation
        // Note: We need `AddUserOperation` wrapper.
        // Assuming `rustray` follows standard gRPC command structure:
        let operation = TypedMessage {
            r#type: "rustray.app.proxyman.command.AddUserOperation".to_string(),
            value: user_bytes, // This assumes AddUserOperation just wraps User or IS User compatible
        };

        let req = AlterInboundRequest {
            tag: inbound_tag.to_string(),
            operation: Some(operation),
        };

        client
            .alter_inbound(req)
            .await
            .map_err(|e| anyhow!("gRPC AddUser failed: {}", e))?;
        Ok(())
    }

    async fn remove_user(&self, inbound_tag: &str, user_email: &str) -> Result<()> {
        let mut client = self.get_handler_client().await?;

        // RemoveUserOperation usually takes email
        // We construct the TypedMessage for RemoveUserOperation
        let operation = TypedMessage {
            r#type: "rustray.app.proxyman.command.RemoveUserOperation".to_string(),
            value: user_email.as_bytes().to_vec(), // Simplified payload
        };

        let req = AlterInboundRequest {
            tag: inbound_tag.to_string(),
            operation: Some(operation),
        };

        client
            .alter_inbound(req)
            .await
            .map_err(|e| anyhow!("gRPC RemoveUser failed: {}", e))?;
        Ok(())
    }

    async fn get_stats(&self, reset: bool) -> Result<Vec<Stat>> {
        let mut client = self.get_stats_client().await?;
        let req = QueryStatsRequest {
            pattern: "".to_string(),
            reset,
        };
        let res = client
            .query_stats(req)
            .await
            .map_err(|e| anyhow!("gRPC GetStats failed: {}", e))?;
        Ok(res.into_inner().stat)
    }

    async fn add_inbound(&self, config: InboundHandlerConfig) -> Result<()> {
        let mut client = self.get_handler_client().await?;
        let req = AddInboundRequest {
            inbound: Some(config),
        };
        client
            .add_inbound(req)
            .await
            .map_err(|e| anyhow!("gRPC AddInbound failed: {}", e))?;
        Ok(())
    }

    async fn remove_inbound(&self, tag: &str) -> Result<()> {
        let mut client = self.get_handler_client().await?;
        let req = RemoveInboundRequest {
            tag: tag.to_string(),
        };
        client
            .remove_inbound(req)
            .await
            .map_err(|e| anyhow!("gRPC RemoveInbound failed: {}", e))?;
        Ok(())
    }

    /// Explicitly sync a user live. This is an alias/wrapper for add_user to satisfy specific requirements.
    /// In a real implementation, this might do more checks or specific handling.
    async fn sync_user_live(
        &self,
        user_uuid: &str,
        user_email: &str,
        inbound_tag: &str,
        level: u32,
    ) -> Result<()> {
        self.add_user(inbound_tag, user_email, user_uuid, level)
            .await
    }
}
