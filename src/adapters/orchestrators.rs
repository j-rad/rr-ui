// src/adapters/orchestrators.rs - Core Orchestrator Adapters
//
// Adapters that communicate with the running core via gRPC

use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ports::CoreOrchestrator as CoreOrchestratorPort;
use crate::models::Inbound;
use crate::services::orchestrator::CoreOrchestrator as InfraOrchestratorTrait;
use crate::services::orchestrator::Orchestrator as InfraOrchestrator;
use async_trait::async_trait;
use std::sync::Arc;

/// Adapter that wraps the infrastructure orchestrator
pub struct GrpcCoreOrchestrator {
    orchestrator: Arc<InfraOrchestrator>,
}

impl GrpcCoreOrchestrator {
    pub fn new(orchestrator: Arc<InfraOrchestrator>) -> Self {
        Self { orchestrator }
    }
}

#[async_trait]
impl CoreOrchestratorPort for GrpcCoreOrchestrator {
    async fn add_user(
        &self,
        inbound_tag: &str,
        email: &str,
        uuid: &str,
        level: u32,
    ) -> DomainResult<()> {
        self.orchestrator
            .add_user(inbound_tag, email, uuid, level)
            .await
            .map_err(|e| DomainError::ExternalServiceError {
                service: "CoreOrchestrator".to_string(),
                message: format!("Failed to add user: {}", e),
            })
    }

    async fn remove_user(&self, inbound_tag: &str, email: &str) -> DomainResult<()> {
        self.orchestrator
            .remove_user(inbound_tag, email)
            .await
            .map_err(|e| DomainError::ExternalServiceError {
                service: "CoreOrchestrator".to_string(),
                message: format!("Failed to remove user: {}", e),
            })
    }

    async fn sync_inbound(&self, inbound: &Inbound) -> DomainResult<()> {
        // For now, syncing an inbound means adding all its users
        if let Some(clients) = inbound.settings.clients() {
            for client in clients {
                if client.enable {
                    let email = match &client.email {
                        Some(e) => e,
                        None => continue,
                    };
                    let id = match &client.id {
                        Some(i) => i,
                        None => continue,
                    };

                    self.add_user(
                        &inbound.tag,
                        email,
                        id,
                        0, // Default level for missing field
                    )
                    .await?;
                }
            }
        }
        Ok(())
    }
}

/// No-op orchestrator for testing
pub struct NoOpOrchestrator;

impl NoOpOrchestrator {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CoreOrchestratorPort for NoOpOrchestrator {
    async fn add_user(
        &self,
        _inbound_tag: &str,
        _email: &str,
        _uuid: &str,
        _level: u32,
    ) -> DomainResult<()> {
        Ok(())
    }

    async fn remove_user(&self, _inbound_tag: &str, _email: &str) -> DomainResult<()> {
        Ok(())
    }

    async fn sync_inbound(&self, _inbound: &Inbound) -> DomainResult<()> {
        Ok(())
    }
}
