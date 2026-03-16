// src/domain/ports.rs - Domain Ports (Repository Traits)
//
// These traits define the contract for data access without any knowledge
// of the underlying storage mechanism (SurrealDB, files, in-memory, etc.)

use crate::domain::errors::DomainResult;
use crate::models::{AllSetting, Client, ClientTraffic, Inbound, OutboundModel};
use async_trait::async_trait;

/// Port for inbound configuration persistence
#[async_trait]
pub trait InboundRepository: Send + Sync {
    /// Retrieve all inbounds
    async fn find_all(&self) -> DomainResult<Vec<Inbound<'static>>>;

    /// Find inbound by ID
    async fn find_by_id(&self, id: &str) -> DomainResult<Option<Inbound<'static>>>;

    /// Find inbound by tag (unique identifier)
    async fn find_by_tag(&self, tag: &str) -> DomainResult<Option<Inbound<'static>>>;

    /// Find all enabled inbounds
    async fn find_enabled(&self) -> DomainResult<Vec<Inbound<'static>>>;

    /// Create a new inbound
    async fn create(&self, inbound: Inbound<'static>) -> DomainResult<Inbound<'static>>;

    /// Update an existing inbound
    async fn update(&self, id: &str, inbound: Inbound<'static>) -> DomainResult<Inbound<'static>>;

    /// Delete an inbound
    async fn delete(&self, id: &str) -> DomainResult<Inbound<'static>>;

    /// Check if a tag exists
    async fn tag_exists(&self, tag: &str) -> DomainResult<bool>;

    /// Check if a port is in use
    async fn port_in_use(&self, port: u32) -> DomainResult<bool>;
}

/// Port for outbound configuration persistence
#[async_trait]
pub trait OutboundRepository: Send + Sync {
    /// Retrieve all outbounds
    async fn find_all(&self) -> DomainResult<Vec<OutboundModel<'static>>>;

    /// Find outbound by ID
    async fn find_by_id(&self, id: &str) -> DomainResult<Option<OutboundModel<'static>>>;

    /// Find outbound by tag
    async fn find_by_tag(&self, tag: &str) -> DomainResult<Option<OutboundModel<'static>>>;

    /// Find all enabled outbounds
    async fn find_enabled(&self) -> DomainResult<Vec<OutboundModel<'static>>>;
}

/// Port for user/client data and traffic management
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Get traffic stats for a specific user
    async fn get_traffic(&self, email: &str) -> DomainResult<Option<ClientTraffic>>;

    /// Reset traffic counters for a user
    async fn reset_traffic(&self, email: &str) -> DomainResult<()>;

    /// Get all users with traffic data
    async fn get_all_traffic(&self) -> DomainResult<Vec<ClientTraffic>>;

    /// Update traffic counters
    async fn update_traffic(&self, email: &str, up: i64, down: i64) -> DomainResult<()>;

    /// Find client by email across all inbounds
    async fn find_client_by_email(&self, email: &str) -> DomainResult<Option<(Client, String)>>;

    /// Check if client email is unique across all enabled inbounds
    async fn email_is_unique(
        &self,
        email: &str,
        exclude_inbound: Option<&str>,
    ) -> DomainResult<bool>;
}

/// Port for application settings
#[async_trait]
pub trait SettingRepository: Send + Sync {
    /// Get current settings
    async fn get(&self) -> DomainResult<Option<AllSetting>>;

    async fn save(&self, settings: AllSetting) -> DomainResult<()>;

    /// Update core type setting
    async fn update_core_type(&self, core_type: String) -> DomainResult<()>;
}

/// Port for configuration validation (external service)
#[async_trait]
pub trait ConfigValidator: Send + Sync {
    /// Validate a configuration against the running core
    async fn validate(&self, config_json: &str) -> DomainResult<()>;
}

/// Port for core orchestration (external service)
#[async_trait]
pub trait CoreOrchestrator: Send + Sync {
    /// Add a user to the running core
    async fn add_user(
        &self,
        inbound_tag: &str,
        email: &str,
        uuid: &str,
        level: u32,
    ) -> DomainResult<()>;

    /// Remove a user from the running core
    async fn remove_user(&self, inbound_tag: &str, email: &str) -> DomainResult<()>;

    /// Sync entire inbound to core
    async fn sync_inbound(&self, inbound: &Inbound<'static>) -> DomainResult<()>;
}

/// Port for GeoIP/GeoSite data access
#[async_trait]
pub trait GeoRepository: Send + Sync {
    /// Check if an IP belongs to a country/list
    async fn lookup_ip(&self, ip: std::net::IpAddr) -> DomainResult<Option<String>>;

    /// Check if a domain belongs to a list
    async fn lookup_domain(&self, domain: &str) -> DomainResult<Option<String>>;

    /// Reload databases
    async fn reload(&self) -> DomainResult<()>;
}
