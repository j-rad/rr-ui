// src/adapters/validators.rs - Configuration Validator Adapters
//
// Adapters that validate configurations using external services

use crate::db::DbClient;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ports::ConfigValidator;
use crate::rustray_config::RustRayConfig;
use crate::rustray_process::validate_config;
use async_trait::async_trait;

/// Validator adapter that uses rustray_process::validate_config
pub struct RustRayConfigValidator {
    db: DbClient,
}

impl RustRayConfigValidator {
    pub fn new(db: DbClient) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ConfigValidator for RustRayConfigValidator {
    async fn validate(&self, config_json: &str) -> DomainResult<()> {
        // Parse the config
        let config: RustRayConfig =
            serde_json::from_str(config_json).map_err(|e| DomainError::ValidationFailed {
                field: "config".to_string(),
                reason: format!("Invalid JSON: {}", e),
            })?;

        // Use existing validate_config function
        validate_config(&config, &self.db)
            .await
            .map_err(|e| DomainError::ConfigurationError {
                message: format!("Configuration validation failed: {}", e),
            })
    }
}

/// No-op validator for testing
pub struct NoOpValidator;

impl NoOpValidator {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ConfigValidator for NoOpValidator {
    async fn validate(&self, _config_json: &str) -> DomainResult<()> {
        Ok(())
    }
}
