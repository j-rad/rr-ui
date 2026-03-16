// src/domain/errors.rs - Domain-specific error types
//
// Clean error types that are independent of infrastructure.
// Infrastructure errors are mapped to these at the adapter boundary.

use std::fmt;

/// Core domain error type - infrastructure-agnostic
#[derive(Debug, Clone)]
pub enum DomainError {
    /// Resource not found (e.g., inbound, user)
    NotFound { resource: String, id: String },

    /// Validation failed
    ValidationFailed { field: String, reason: String },

    /// Business rule violation
    BusinessRuleViolation { rule: String, details: String },

    /// Conflict (e.g., duplicate tag, port already in use)
    Conflict { message: String },

    /// Repository/storage error (mapped from infrastructure)
    RepositoryError { message: String },

    /// External service error (e.g., gRPC call failed)
    ExternalServiceError { service: String, message: String },

    /// Configuration error
    ConfigurationError { message: String },
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::NotFound { resource, id } => {
                write!(f, "{} with id '{}' not found", resource, id)
            }
            DomainError::ValidationFailed { field, reason } => {
                write!(f, "Validation failed for field '{}': {}", field, reason)
            }
            DomainError::BusinessRuleViolation { rule, details } => {
                write!(f, "Business rule '{}' violated: {}", rule, details)
            }
            DomainError::Conflict { message } => {
                write!(f, "Conflict: {}", message)
            }
            DomainError::RepositoryError { message } => {
                write!(f, "Repository error: {}", message)
            }
            DomainError::ExternalServiceError { service, message } => {
                write!(f, "External service '{}' error: {}", service, message)
            }
            DomainError::ConfigurationError { message } => {
                write!(f, "Configuration error: {}", message)
            }
        }
    }
}

impl std::error::Error for DomainError {}

/// Result type using DomainError
pub type DomainResult<T> = Result<T, DomainError>;

/// Helper to map anyhow errors to DomainError::RepositoryError
impl From<anyhow::Error> for DomainError {
    fn from(err: anyhow::Error) -> Self {
        DomainError::RepositoryError {
            message: err.to_string(),
        }
    }
}

#[cfg(feature = "server")]
impl From<surrealdb::Error> for DomainError {
    fn from(err: surrealdb::Error) -> Self {
        DomainError::RepositoryError {
            message: err.to_string(),
        }
    }
}
