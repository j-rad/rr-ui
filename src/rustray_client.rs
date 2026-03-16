// src/rustray_client.rs
#![cfg(feature = "server")]
use anyhow::{Result, anyhow};
use tonic::transport::Channel;

// Include generated code from proto/rustray.proto
pub mod rustray_proxyman {
    tonic::include_proto!("rustray.core.app.proxyman.command");
}

pub mod grpc_health {
    tonic::include_proto!("grpc.health.v1");
}

use grpc_health::{HealthCheckRequest, health_client::HealthClient};
use rustray_proxyman::{
    AddInboundRequest, InboundHandlerConfig, QueryStatsRequest, RemoveInboundRequest, Stat,
    handler_service_client::HandlerServiceClient, stats_service_client::StatsServiceClient,
};

/// Client for communicating with the local RustRay core via gRPC.
///
/// This client manages connections to RustRay's API services (HandlerService, StatsService, etc.).
#[derive(Clone)]
pub struct RustRayClient {
    /// Client for HandlerService (add/remove inbounds).
    handler_client: Option<HandlerServiceClient<Channel>>,
    /// Client for StatsService (traffic stats).
    stats_client: Option<StatsServiceClient<Channel>>,
    /// Client for HealthService.
    health_client: Option<HealthClient<Channel>>,
    /// Port where RustRay API is listening.
    api_port: u16,
}

impl RustRayClient {
    /// Creates a new RustRayClient instance.
    pub fn new(api_port: u16) -> Self {
        Self {
            handler_client: None,
            stats_client: None,
            health_client: None,
            api_port,
        }
    }

    /// Establishes a gRPC connection to the RustRay core with exponential backoff retry.
    ///
    /// Retries up to 5 times with exponential backoff (1s, 2s, 4s, 8s, 16s).
    pub async fn connect(&mut self) -> Result<()> {
        self.connect_with_retry(5).await
    }

    /// Establishes a gRPC connection with custom retry count.
    pub async fn connect_with_retry(&mut self, max_retries: u32) -> Result<()> {
        let addr = format!("http://127.0.0.1:{}", self.api_port);
        let mut retry_count = 0;
        let mut delay_ms = 1000; // Start with 1 second

        loop {
            match Channel::from_shared(addr.clone())?.connect().await {
                Ok(channel) => {
                    self.handler_client = Some(HandlerServiceClient::new(channel.clone()));
                    self.stats_client = Some(StatsServiceClient::new(channel.clone()));
                    self.health_client = Some(HealthClient::new(channel));
                    log::info!("Successfully connected to RustRay gRPC API at {}", addr);
                    return Ok(());
                }
                Err(e) => {
                    retry_count += 1;
                    if retry_count >= max_retries {
                        return Err(anyhow!(
                            "Failed to connect to RustRay gRPC at {} after {} retries: {}",
                            addr,
                            max_retries,
                            e
                        ));
                    }

                    log::warn!(
                        "RustRay gRPC connection failed (attempt {}/{}): {}. Retrying in {}ms...",
                        retry_count,
                        max_retries,
                        e,
                        delay_ms
                    );

                    tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;

                    // Exponential backoff with cap at 16 seconds
                    delay_ms = std::cmp::min(delay_ms * 2, 16000);
                }
            }
        }
    }

    /// Adds a new inbound to the running RustRay instance.
    ///
    /// Note: `inbound_config_bytes` should be a serialized protobuf message of `InboundHandlerConfig`.
    pub async fn add_inbound(&mut self, inbound_config: InboundHandlerConfig) -> Result<()> {
        let client = self
            .handler_client
            .as_mut()
            .ok_or_else(|| anyhow!("Not connected"))?;

        let request = AddInboundRequest {
            inbound: Some(inbound_config),
        };

        client
            .add_inbound(request)
            .await
            .map_err(|e| anyhow!("gRPC AddInbound failed: {}", e))?;
        Ok(())
    }

    /// Removes an inbound by its tag.
    pub async fn remove_inbound(&mut self, tag: String) -> Result<()> {
        let client = self
            .handler_client
            .as_mut()
            .ok_or_else(|| anyhow!("Not connected"))?;
        client.remove_inbound(RemoveInboundRequest { tag }).await?;
        Ok(())
    }

    /// Retrieves traffic statistics (upload/download) for all users/inbounds.
    ///
    /// If `reset` is true, counters will be reset after reading.
    pub async fn get_traffic_stats(&mut self, reset: bool) -> Result<Vec<Stat>> {
        let client = self
            .stats_client
            .as_mut()
            .ok_or_else(|| anyhow!("Not connected"))?;
        // Empty pattern matches all stats
        let req = QueryStatsRequest {
            pattern: "".to_string(),
            reset,
        };
        let response = client.query_stats(req).await?;
        Ok(response.into_inner().stat)
    }
    /// Checks if the gRPC clients are initialized and potentially healthy.
    pub fn is_healthy(&self) -> bool {
        self.handler_client.is_some() && self.stats_client.is_some()
    }

    /// Returns the port where RustRay API is listening.
    pub fn api_port(&self) -> u16 {
        self.api_port
    }

    /// Performs a standard gRPC health check.
    pub async fn check_health(&mut self) -> Result<()> {
        let client = self
            .health_client
            .as_mut()
            .ok_or_else(|| anyhow!("Not connected"))?;

        let request = HealthCheckRequest {
            service: "".to_string(),
        };

        let response = client
            .check(request)
            .await
            .map_err(|e| anyhow!("gRPC Health Check failed: {}", e))?;

        if response.into_inner().status == 1 {
            // SERVING
            Ok(())
        } else {
            Err(anyhow!("gRPC Health Status: NOT_SERVING"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_is_healthy_initially_false() {
        let client = RustRayClient::new(10085);
        // Initially, the client should not be healthy
        assert!(!client.is_healthy());
        // Port should be set correctly
        assert_eq!(client.api_port(), 10085);
    }

    #[tokio::test]
    async fn test_client_grpc_heartbeat_unconnected() {
        let mut client = RustRayClient::new(10085);
        // The heartbeat relies on get_traffic_stats. If unconnected, it must fail cleanly.
        let result = client.get_traffic_stats(false).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Not connected");
    }
}
