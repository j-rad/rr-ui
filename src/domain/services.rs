// src/domain/services.rs - Domain Services (Business Logic)
//
// These services contain the core business logic and orchestrate operations
// using the repository ports. They have NO direct dependencies on infrastructure.

use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ports::{
    ConfigValidator, CoreOrchestrator, InboundRepository, OutboundRepository, UserRepository,
};
use crate::models::{ClientTraffic, Inbound, ProtocolSettings};
use crate::rustray_config::RustRayConfigBuilder;
use std::collections::HashSet;

/// Service for managing inbound configurations
pub struct InboundService<R: InboundRepository, O: OutboundRepository> {
    inbound_repo: R,
    outbound_repo: O,
}

impl<R: InboundRepository, O: OutboundRepository> InboundService<R, O> {
    pub fn new(inbound_repo: R, outbound_repo: O) -> Self {
        Self {
            inbound_repo,
            outbound_repo,
        }
    }

    /// List all inbounds
    pub async fn list_all(&self) -> DomainResult<Vec<Inbound<'static>>> {
        self.inbound_repo.find_all().await
    }

    /// Get inbound by ID
    pub async fn get_by_id(&self, id: &str) -> DomainResult<Inbound<'static>> {
        self.inbound_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| DomainError::NotFound {
                resource: "Inbound".to_string(),
                id: id.to_string(),
            })
    }

    /// Get inbound by tag
    pub async fn get_by_tag(&self, tag: &str) -> DomainResult<Inbound<'static>> {
        self.inbound_repo
            .find_by_tag(tag)
            .await?
            .ok_or_else(|| DomainError::NotFound {
                resource: "Inbound".to_string(),
                id: tag.to_string(),
            })
    }

    /// Create a new inbound with validation
    pub async fn create<V: ConfigValidator + ?Sized>(
        &self,
        inbound: Inbound<'static>,
        validator: &V,
    ) -> DomainResult<Inbound<'static>> {
        // Business rule: Tag must be unique
        if self.inbound_repo.tag_exists(&inbound.tag).await? {
            return Err(DomainError::Conflict {
                message: format!("Inbound tag '{}' already exists", inbound.tag),
            });
        }

        // Business rule: Port must not be in use
        if self.inbound_repo.port_in_use(inbound.port).await? {
            return Err(DomainError::Conflict {
                message: format!("Port {} is already in use", inbound.port),
            });
        }

        // Validation: Tag must not be empty
        if inbound.tag.trim().is_empty() {
            return Err(DomainError::ValidationFailed {
                field: "tag".to_string(),
                reason: "Tag cannot be empty".to_string(),
            });
        }

        // Validation: Port must be in valid range
        if inbound.port == 0 || inbound.port > 65535 {
            return Err(DomainError::ValidationFailed {
                field: "port".to_string(),
                reason: format!("Port {} is invalid (must be 1-65535)", inbound.port),
            });
        }

        // Validation: Protocol must be supported
        self.validate_protocol(&inbound.settings)?;

        // Validation: Simulate full config to ensure it's valid
        self.validate_with_config(&inbound, validator).await?;

        // Create in repository
        self.inbound_repo.create(inbound).await
    }

    /// Update an existing inbound with validation
    pub async fn update<V: ConfigValidator + ?Sized>(
        &self,
        id: &str,
        inbound: Inbound<'static>,
        validator: &V,
    ) -> DomainResult<Inbound<'static>> {
        // Ensure the inbound exists
        let existing = self.get_by_id(id).await?;

        // Business rule: If tag changed, new tag must be unique
        if existing.tag != inbound.tag && self.inbound_repo.tag_exists(&inbound.tag).await? {
            return Err(DomainError::Conflict {
                message: format!("Inbound tag '{}' already exists", inbound.tag),
            });
        }

        // Business rule: If port changed, new port must not be in use
        if existing.port != inbound.port && self.inbound_repo.port_in_use(inbound.port).await? {
            return Err(DomainError::Conflict {
                message: format!("Port {} is already in use", inbound.port),
            });
        }

        // Validation: Same as create
        if inbound.tag.trim().is_empty() {
            return Err(DomainError::ValidationFailed {
                field: "tag".to_string(),
                reason: "Tag cannot be empty".to_string(),
            });
        }

        if inbound.port == 0 || inbound.port > 65535 {
            return Err(DomainError::ValidationFailed {
                field: "port".to_string(),
                reason: format!("Port {} is invalid", inbound.port),
            });
        }

        self.validate_protocol(&inbound.settings)?;

        // Validation: Simulate full config
        self.validate_with_config(&inbound, validator).await?;

        // Update in repository
        self.inbound_repo.update(id, inbound).await
    }

    /// Delete an inbound
    pub async fn delete(&self, id: &str) -> DomainResult<Inbound<'static>> {
        // Ensure exists before deleting
        let _ = self.get_by_id(id).await?;

        self.inbound_repo.delete(id).await
    }

    /// Validate protocol settings
    fn validate_protocol(&self, settings: &ProtocolSettings<'_>) -> DomainResult<()> {
        match settings {
            ProtocolSettings::Vless(s) => {
                if s.clients.is_empty() {
                    return Err(DomainError::ValidationFailed {
                        field: "clients".to_string(),
                        reason: "At least one client must be configured".to_string(),
                    });
                }
                self.validate_client_emails(&s.clients)?;
            }
            ProtocolSettings::Vmess(s) => {
                if s.clients.is_empty() {
                    return Err(DomainError::ValidationFailed {
                        field: "clients".to_string(),
                        reason: "At least one client must be configured".to_string(),
                    });
                }
                self.validate_client_emails(&s.clients)?;
            }
            ProtocolSettings::Trojan(s) => {
                if s.clients.is_empty() {
                    return Err(DomainError::ValidationFailed {
                        field: "clients".to_string(),
                        reason: "At least one client must be configured".to_string(),
                    });
                }
                self.validate_client_emails(&s.clients)?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Validate client emails are unique within the inbound
    fn validate_client_emails(&self, clients: &[crate::models::Client]) -> DomainResult<()> {
        let mut seen = HashSet::new();
        for client in clients {
            let email = client.email.as_deref().unwrap_or("");
            if !seen.insert(email) {
                return Err(DomainError::ValidationFailed {
                    field: "clients.email".to_string(),
                    reason: format!("Duplicate email '{}' in clients", email),
                });
            }
            if email.trim().is_empty() {
                return Err(DomainError::ValidationFailed {
                    field: "clients.email".to_string(),
                    reason: "Client email cannot be empty".to_string(),
                });
            }
        }
        Ok(())
    }

    /// Validate by building a full config and validating with the core
    async fn validate_with_config<V: ConfigValidator + ?Sized>(
        &self,
        new_inbound: &Inbound<'static>,
        validator: &V,
    ) -> DomainResult<()> {
        // Fetch existing in bounds and outbounds
        let mut inbounds = self.inbound_repo.find_all().await?;
        let outbounds = self.outbound_repo.find_all().await?;

        // Replace or add the new inbound
        if let Some(pos) = inbounds.iter().position(|i| {
            i.id.as_ref()
                .and_then(|id| new_inbound.id.as_ref().map(|new_id| id == new_id))
                .unwrap_or(false)
        }) {
            inbounds[pos] = new_inbound.clone();
        } else {
            inbounds.push(new_inbound.clone());
        }

        // Build full config
        let config = RustRayConfigBuilder::build_from_models(&inbounds, &outbounds);
        let config_json =
            serde_json::to_string(&config).map_err(|e| DomainError::ConfigurationError {
                message: format!("Failed to serialize config: {}", e),
            })?;

        // Validate using the external validator
        validator.validate(&config_json).await
    }

    /// Sync entire inbound to core
    pub async fn sync_inbound<C: CoreOrchestrator>(
        &self,
        inbound: &Inbound<'static>,
        orchestrator: &C,
    ) -> DomainResult<()> {
        orchestrator.sync_inbound(inbound).await
    }
}

/// Service for managing users and traffic
pub struct UserService<U: UserRepository, I: InboundRepository> {
    user_repo: U,
    _inbound_repo: I,
}

impl<U: UserRepository, I: InboundRepository> UserService<U, I> {
    pub fn new(user_repo: U, inbound_repo: I) -> Self {
        Self {
            user_repo,
            _inbound_repo: inbound_repo,
        }
    }

    /// Get traffic for a user
    pub async fn get_traffic(&self, email: &str) -> DomainResult<ClientTraffic> {
        self.user_repo
            .get_traffic(email)
            .await?
            .ok_or_else(|| DomainError::NotFound {
                resource: "User traffic".to_string(),
                id: email.to_string(),
            })
    }

    /// Reset traffic for a user
    pub async fn reset_traffic(&self, email: &str) -> DomainResult<()> {
        // Ensure user exists
        let _ = self.get_traffic(email).await?;

        self.user_repo.reset_traffic(email).await
    }

    /// Get all user traffic
    pub async fn get_all_traffic(&self) -> DomainResult<Vec<ClientTraffic>> {
        self.user_repo.get_all_traffic().await
    }

    /// Sync a user to the running core
    pub async fn sync_user<C: CoreOrchestrator>(
        &self,
        email: &str,
        orchestrator: &C,
    ) -> DomainResult<()> {
        // Find the user across all inbounds
        let (client, inbound_tag) = self
            .user_repo
            .find_client_by_email(email)
            .await?
            .ok_or_else(|| DomainError::NotFound {
                resource: "User".to_string(),
                id: email.to_string(),
            })?;

        // Sync to core - use default level 0 since Client doesn't have a level field
        orchestrator
            .add_user(
                &inbound_tag,
                &client.email.as_deref().unwrap_or(""),
                &client.id.as_deref().unwrap_or(""),
                0, // Default level
            )
            .await
    }
}
