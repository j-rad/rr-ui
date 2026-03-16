// src/domain/proxy_core.rs
//! ProxyCore Trait
//!
//! Abstract interface for proxy backends (RustRay, Sing-box, etc.)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Proxy core trait
///
/// All proxy backends must implement this trait
#[async_trait::async_trait]
pub trait ProxyCore: Send + Sync {
    /// Get core name (e.g., "rustray", "sing-box")
    fn name(&self) -> &'static str;

    /// Get core version
    async fn version(&self) -> Result<String, CoreError>;

    /// Start the proxy core
    async fn start(&mut self, config: CoreConfig) -> Result<(), CoreError>;

    /// Stop the proxy core
    async fn stop(&mut self) -> Result<(), CoreError>;

    /// Restart the proxy core
    async fn restart(&mut self) -> Result<(), CoreError> {
        self.stop().await?;
        // Config is stored internally
        Ok(())
    }

    /// Check if core is running
    fn is_running(&self) -> bool;

    /// Get supported protocols
    fn supported_protocols(&self) -> Vec<Protocol>;

    /// Validate configuration
    async fn validate_config(&self, config: &CoreConfig) -> Result<(), CoreError>;

    /// Get real-time statistics
    async fn get_stats(&self) -> Result<CoreStats, CoreError>;

    /// Add user dynamically
    async fn add_user(&mut self, user: UserConfig) -> Result<(), CoreError>;

    /// Remove user dynamically
    async fn remove_user(&mut self, user_id: &str) -> Result<(), CoreError>;

    /// Update user configuration
    async fn update_user(&mut self, user_id: &str, user: UserConfig) -> Result<(), CoreError>;
}

/// Core configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreConfig {
    pub log_level: LogLevel,
    pub inbounds: Vec<InboundConfig>,
    pub outbounds: Vec<OutboundConfig>,
    pub routing: RoutingConfig,
    pub dns: Option<DnsConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    None,
}

/// Inbound configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundConfig {
    pub tag: String,
    pub protocol: Protocol,
    pub listen: String,
    pub port: u16,
    pub settings: serde_json::Value,
}

/// Outbound configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboundConfig {
    pub tag: String,
    pub protocol: Protocol,
    pub settings: serde_json::Value,
}

/// Routing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    pub rules: Vec<RoutingRule>,
    pub domain_strategy: DomainStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    pub inbound_tag: Option<String>,
    pub outbound_tag: String,
    pub domain: Option<Vec<String>>,
    pub ip: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DomainStrategy {
    AsIs,
    IpIfNonMatch,
    IpOnDemand,
}

/// DNS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsConfig {
    pub servers: Vec<String>,
    pub hosts: HashMap<String, String>,
}

/// Supported protocols
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Vmess,
    Vless,
    Trojan,
    Shadowsocks,
    Hysteria2,
    Tuic,
    Socks,
    Http,
    Freedom,
    Blackhole,
}

impl Protocol {
    pub fn display_name(&self) -> &'static str {
        match self {
            Protocol::Vmess => "VMess",
            Protocol::Vless => "VLESS",
            Protocol::Trojan => "Trojan",
            Protocol::Shadowsocks => "Shadowsocks",
            Protocol::Hysteria2 => "Hysteria2",
            Protocol::Tuic => "TUIC",
            Protocol::Socks => "SOCKS",
            Protocol::Http => "HTTP",
            Protocol::Freedom => "Freedom",
            Protocol::Blackhole => "Blackhole",
        }
    }
}

/// User configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub id: String,
    pub email: String,
    pub uuid: String,
    pub level: u8,
    pub alter_id: Option<u16>,
}

/// Core statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreStats {
    pub uptime_seconds: u64,
    pub total_upload_bytes: u64,
    pub total_download_bytes: u64,
    pub active_connections: usize,
    pub user_stats: HashMap<String, UserStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStats {
    pub upload_bytes: u64,
    pub download_bytes: u64,
    pub last_active: i64,
}

/// Core errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoreError {
    NotRunning,
    AlreadyRunning,
    ConfigInvalid(String),
    StartFailed(String),
    StopFailed(String),
    UserNotFound(String),
    ProtocolNotSupported(String),
    ApiError(String),
}

impl std::fmt::Display for CoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoreError::NotRunning => write!(f, "Core is not running"),
            CoreError::AlreadyRunning => write!(f, "Core is already running"),
            CoreError::ConfigInvalid(msg) => write!(f, "Invalid configuration: {}", msg),
            CoreError::StartFailed(msg) => write!(f, "Failed to start core: {}", msg),
            CoreError::StopFailed(msg) => write!(f, "Failed to stop core: {}", msg),
            CoreError::UserNotFound(id) => write!(f, "User not found: {}", id),
            CoreError::ProtocolNotSupported(proto) => {
                write!(f, "Protocol not supported: {}", proto)
            }
            CoreError::ApiError(msg) => write!(f, "API error: {}", msg),
        }
    }
}

impl std::error::Error for CoreError {}

/// Core factory
pub struct CoreFactory;

impl CoreFactory {
    /// Create a proxy core by name
    pub fn create(name: &str) -> Result<Box<dyn ProxyCore>, String> {
        match name {
            "rustray" => {
                #[cfg(feature = "server")]
                {
                    Ok(Box::new(
                        super::super::adapters::rustray_core::RustRayCore::new(),
                    ))
                }
                #[cfg(not(feature = "server"))]
                {
                    Err("RustRay core requires server feature".to_string())
                }
            }
            _ => Err(format!("Unknown core: {}", name)),
        }
    }

    /// List available cores
    pub fn available_cores() -> Vec<&'static str> {
        vec!["rustray"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_display_name() {
        assert_eq!(Protocol::Vmess.display_name(), "VMess");
        assert_eq!(Protocol::Hysteria2.display_name(), "Hysteria2");
    }

    #[test]
    fn test_core_error_display() {
        let err = CoreError::NotRunning;
        assert_eq!(err.to_string(), "Core is not running");
    }

    #[test]
    fn test_available_cores() {
        let cores = CoreFactory::available_cores();
        assert!(cores.contains(&"rustray"));
    }
}
