// src/domain/models.rs - Unified Domain Models
//
// This module contains all domain entities shared between the server and client.
// All types are designed to be serializable and work across the Dioxus Fullstack architecture.
//
// Key Design Decisions:
// - All types use `#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]` for compatibility
// - SecretString is feature-gated for server-only use; client uses plain String
// - Cow<'a, str> is used for zero-copy deserialization on server
// - Validation logic is implemented via the validator crate where applicable

use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;

#[cfg(feature = "server")]
use secrecy::{ExposeSecret, SecretString};
#[cfg(feature = "server")]
#[cfg(feature = "server")]
use surrealdb::sql::Thing;
#[cfg(feature = "server")]
use validator::Validate;

// Include the external test module
#[cfg(test)]
#[path = "models_tests.rs"]
mod tests;

// ============================================================================
// Type Aliases for Feature-Gated Types
// ============================================================================

/// ID type that varies based on feature flags
#[cfg(feature = "server")]
pub type IdType = Thing;
#[cfg(not(feature = "server"))]
pub type IdType = String;

/// Secret type for sensitive data - server uses SecretString, client uses String
/// Secret type for sensitive data - for now using String on both sides to satisfy PartialEq
#[cfg(feature = "server")]
pub type SecretType = String;
#[cfg(not(feature = "server"))]
pub type SecretType = String;

// ============================================================================
// Serialization Helpers
// ============================================================================

#[cfg(feature = "server")]
fn serialize_secret<S>(secret: &SecretString, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(secret.expose_secret())
}

#[cfg(feature = "server")]
fn serialize_secret_option<S>(
    secret: &Option<SecretString>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match secret {
        Some(s) => serializer.serialize_str(s.expose_secret()),
        None => serializer.serialize_none(),
    }
}

fn is_cow_empty_str(c: &Cow<'_, str>) -> bool {
    c.is_empty()
}

// ============================================================================
// API Response Types
// ============================================================================

/// Standard response wrapper for all API endpoints.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ApiResponse<T = serde_json::Value> {
    pub success: bool,
    pub msg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub obj: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn success(msg: impl Into<String>, obj: Option<T>) -> Self {
        Self {
            success: true,
            msg: msg.into(),
            obj,
        }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            msg: msg.into(),
            obj: None,
        }
    }
}

// ============================================================================
// Authentication Types
// ============================================================================

/// User roles for RBAC
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    #[default]
    Admin,
    Reseller,
}

/// User entity for authentication
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct User {
    pub id: Option<i64>,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<UserRole>,
}

/// Payload for login requests.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub two_factor_code: Option<String>,
}

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

// ============================================================================
// System Status Types
// ============================================================================

/// Current/Total value pair
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct CurTotal {
    pub current: u64,
    pub total: u64,
}

/// Network I/O rates
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct NetIO {
    pub up: u64,
    pub down: u64,
}

/// Network traffic counters
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct NetTraffic {
    pub sent: u64,
    pub recv: u64,
}

/// A single point in traffic history
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct TrafficHistoryPoint {
    pub timestamp: u64,
    pub up_rate: u64,   // Bytes per second
    pub down_rate: u64, // Bytes per second
}

/// Public IP addresses
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct PublicIP {
    pub ipv4: String,
    pub ipv6: String,
}

/// RustRay core state
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RustRayState {
    pub state: RustRayStatus,
    #[serde(rename = "errorMsg")]
    pub error_msg: String,
    pub version: String,
}

impl Default for RustRayState {
    fn default() -> Self {
        Self {
            state: RustRayStatus::Stop,
            error_msg: String::new(),
            version: String::new(),
        }
    }
}

/// RustRay process status enum
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RustRayStatus {
    Running,
    Stop,
    Error,
}

/// Application-specific statistics
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppStats {
    pub threads: u32,
    pub mem: u64,
    pub uptime: u64,
}

/// Comprehensive server status
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ServerStatus {
    pub cpu: f64,
    pub cpu_cores: u32,
    pub logical_pro: u32,
    pub cpu_speed_mhz: f64,
    pub mem: CurTotal,
    pub swap: CurTotal,
    pub disk: CurTotal,
    pub loads: [f64; 3],
    #[serde(rename = "netIO")]
    pub net_io: NetIO,
    pub net_traffic: NetTraffic,
    #[serde(rename = "publicIP")]
    pub public_ip: PublicIP,
    pub tcp_count: u32,
    pub udp_count: u32,
    pub uptime: u64,
    pub app_stats: AppStats,
    pub rustray: RustRayState,
    pub active_protocols: Vec<String>,
}

impl Default for ServerStatus {
    fn default() -> Self {
        Self {
            cpu: 0.0,
            cpu_cores: 0,
            logical_pro: 0,
            cpu_speed_mhz: 0.0,
            mem: CurTotal::default(),
            swap: CurTotal::default(),
            disk: CurTotal::default(),
            loads: [0.0, 0.0, 0.0],
            net_io: NetIO::default(),
            net_traffic: NetTraffic::default(),
            public_ip: PublicIP::default(),
            tcp_count: 0,
            udp_count: 0,
            uptime: 0,
            app_stats: AppStats::default(),
            rustray: RustRayState::default(),
            active_protocols: Vec::new(),
        }
    }
}

/// Real-time telemetry broadcast message
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct RealtimeTelemetry {
    pub system: Option<serde_json::Value>, // SystemTelemetry is in services/telemetry.rs, keeping it generic here or moving it
    pub traffic: NetIO,
    pub server_status: Option<ServerStatus>,
}

// ============================================================================
// Client & Traffic Types
// ============================================================================

/// Flow types for XTLS/Vision
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum FlowType {
    #[serde(rename = "")]
    None,
    #[serde(rename = "xtls-rprx-vision")]
    XtlsRprxVision,
    #[serde(rename = "xtls-rprx-vision-udp443")]
    XtlsRprxVisionUdp443,
    #[serde(untagged)]
    Other(String),
}

impl Default for FlowType {
    fn default() -> Self {
        Self::None
    }
}

/// Post-Quantum Cryptography Matrix (Algorithm Levels)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PqcMatrix {
    Kyber768,
    Kyber1024,
    Dilithium2,
    Dilithium3,
    Dilithium5,
    Mceliece6688128,
    #[serde(untagged)]
    Other(String),
}

/// Inbound Protocol Types
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Copy)]
#[serde(rename_all = "lowercase")]
pub enum InboundProtocol {
    Vless,
    Vmess,
    Trojan,
    Shadowsocks,
    Socks,
    Http,
    WireGuard,
    #[serde(rename = "dokodemo-door")]
    Dokodemo,
    #[serde(rename = "flowj")]
    FlowJ,
    #[serde(rename = "tun")]
    Tun,
}

impl ToString for InboundProtocol {
    fn to_string(&self) -> String {
        match self {
            InboundProtocol::Vless => "vless".to_string(),
            InboundProtocol::Vmess => "vmess".to_string(),
            InboundProtocol::Trojan => "trojan".to_string(),
            InboundProtocol::Shadowsocks => "shadowsocks".to_string(),
            InboundProtocol::Socks => "socks".to_string(),
            InboundProtocol::Http => "http".to_string(),
            InboundProtocol::WireGuard => "wireguard".to_string(),
            InboundProtocol::Dokodemo => "dokodemo-door".to_string(),
            InboundProtocol::FlowJ => "flowj".to_string(),
            InboundProtocol::Tun => "tun".to_string(),
        }
    }
}

/// Client configuration within an inbound
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "server", derive(Validate))]
pub struct Client {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>, // UUID for vless/vmess
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>, // For Trojan (note: String for secrecy)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flow: Option<FlowType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "server", validate(email))]
    pub email: Option<String>,
    #[serde(default, alias = "total_gb", alias = "totalGB")]
    pub total_flow_limit: u64,
    #[serde(default, alias = "user_expiry_time", alias = "expiryTime")]
    pub expiry_time: i64,
    #[serde(default)]
    pub enable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tg_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_id: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        alias = "ip_limit",
        alias = "limitIp"
    )]
    #[cfg_attr(feature = "server", validate(range(min = 1, max = 65535)))]
    pub limit_ip: Option<u32>,
    #[serde(default, alias = "upSpeedLimit")]
    pub up_speed_limit: u32,
    #[serde(default, alias = "downSpeedLimit")]
    pub down_speed_limit: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_reset_date: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inbound_tag: Option<String>,
    /// Reset period in days (0 = no reset)
    #[serde(default)]
    pub reset: i32,
    /// Client comment/description
    #[serde(skip_serializing_if = "Option::is_none", alias = "remark")]
    pub comment: Option<String>,
    /// Creation timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    /// Last update timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<i64>,
    #[serde(default)]
    pub up: i64,
    #[serde(default)]
    pub down: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<u32>,
    /// The ID of the reseller who created this client (scoping)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<i64>,
    /// Capture any extra fields for forward compatibility
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Client traffic statistics
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ClientTraffic {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inbound_id: Option<i64>,
    pub enable: bool,
    pub email: String,
    pub up: i64,
    pub down: i64,
    pub expiry_time: i64,
    pub total: i64,
}

impl Default for ClientTraffic {
    fn default() -> Self {
        Self {
            id: None,
            inbound_id: None,
            enable: true,
            email: String::new(),
            up: 0,
            down: 0,
            expiry_time: 0,
            total: 0,
        }
    }
}

/// WireGuard peer configuration
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WireguardPeer {
    pub public_key: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_ips: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(default)]
    pub keep_alive: u32,
}

// ============================================================================
// Live Connections / Sniffer Types
// ============================================================================

/// Active connection information
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    pub ip: String,
    pub domain: String,
    pub protocol: String,
    pub duration: u64,
    pub latency: u64,
    /// Unique connection ID from backend
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Inbound tag this connection belongs to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inbound_tag: Option<String>,
    /// Client email if authenticated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Current upload speed in bytes/sec
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upload_speed: Option<u64>,
    /// Current download speed in bytes/sec
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_speed: Option<u64>,
}

impl Default for Connection {
    fn default() -> Self {
        Self {
            ip: String::new(),
            domain: String::new(),
            protocol: String::new(),
            duration: 0,
            latency: 0,
            id: None,
            inbound_tag: None,
            email: None,
            upload_speed: None,
            download_speed: None,
        }
    }
}

/// Sniffer event types for connection monitoring
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SnifferEventType {
    Open,
    Close,
    Update,
}

/// Sniffer event for real-time connection updates
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SnifferEvent {
    #[serde(rename = "type")]
    pub event_type: SnifferEventType,
    pub connection: Connection,
    pub timestamp: i64,
}

/// Traffic update for an inbound
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TrafficUpdate {
    pub inbound_tag: String,
    pub up: i64,
    pub down: i64,
    /// 60-second history for sparkline visualization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<Vec<i64>>,
}

// ============================================================================
// Protocol-Specific Settings
// ============================================================================

/// VLESS protocol settings
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VlessSettings<'a> {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clients: Vec<Client>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decryption: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallbacks: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// VMess protocol settings
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VmessSettings {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clients: Vec<Client>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallbacks: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Trojan protocol settings
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TrojanSettings {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clients: Vec<Client>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallbacks: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Shadowsocks 2022 protocol settings
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Shadowsocks2022Settings<'a> {
    pub method: Cow<'a, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<u32>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Hysteria2 obfuscation settings
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Obfuscation<'a> {
    #[serde(rename = "type")]
    pub obfs_type: Cow<'a, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

/// Hysteria2 protocol settings
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Hysteria2Settings<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub up_mbps: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub down_mbps: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub obfuscation: Option<Obfuscation<'a>>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// TUIC user configuration
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TuicUser<'a> {
    pub uuid: Cow<'a, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<Cow<'a, str>>,
}

/// Certificate configuration
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Certificate<'a> {
    #[serde(default, skip_serializing_if = "is_cow_empty_str")]
    pub certificate_file: Cow<'a, str>,
    #[serde(default, skip_serializing_if = "is_cow_empty_str")]
    pub key_file: Cow<'a, str>,
}

/// TUIC protocol settings
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TuicSettings<'a> {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub users: Vec<TuicUser<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub congestion_control: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate: Option<Certificate<'a>>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Flow-J mode enum
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FlowJMode {
    #[default]
    Auto,
    Reality,
    Cdn,
    Mqtt,
}

// FlowUser removed in favor of Client

/// MQTT settings for Flow-J
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MqttSettings<'a> {
    pub broker_address: Cow<'a, str>,
    pub upload_topic: Cow<'a, str>,
    pub download_topic: Cow<'a, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(default)]
    pub qos: u8,
}

/// CDN settings for Flow-J
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CdnSettings<'a> {
    pub path: Cow<'a, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<Cow<'a, str>>,
}

/// FEC (Forward Error Correction) settings
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FecSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_fec_data_shards")]
    pub data_shards: usize,
    #[serde(default = "default_fec_parity_shards")]
    pub parity_shards: usize,
}

impl Default for FecSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            data_shards: default_fec_data_shards(),
            parity_shards: default_fec_parity_shards(),
        }
    }
}

fn default_fec_data_shards() -> usize {
    10
}
fn default_fec_parity_shards() -> usize {
    3
}

/// REALITY protocol settings
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RealitySettings<'a> {
    #[serde(default)]
    pub show: bool,
    #[serde(default, skip_serializing_if = "is_cow_empty_str")]
    pub dest: Cow<'a, str>,
    #[serde(default)]
    pub xver: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub server_names: Vec<Cow<'a, str>>,
    #[serde(default, skip_serializing_if = "is_cow_empty_str")]
    pub private_key: Cow<'a, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_client_ver: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_client_ver: Option<Cow<'a, str>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub short_ids: Vec<Cow<'a, str>>,
    #[serde(default, skip_serializing_if = "is_cow_empty_str")]
    pub fingerprint: Cow<'a, str>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "pqcCipher")]
    pub pqc_matrix: Option<PqcMatrix>,
    #[serde(default)]
    pub stealth_handshake: bool,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Flow-J protocol settings
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FlowJSettings<'a> {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clients: Vec<Client>,
    #[serde(default)]
    pub mode: FlowJMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mqtt: Option<MqttSettings<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reality: Option<RealitySettings<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cdn: Option<CdnSettings<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fec: Option<FecSettings>,
    #[serde(default)]
    pub stealth_handshake: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flowj_config: Option<FlowJConfig>, // New Field
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Naive protocol settings (placeholder)
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NaiveSettings {}

/// WireGuard protocol settings
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WireGuardSettings {
    pub secret_key: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub peers: Vec<WireguardPeer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtu: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reserved: Option<[u8; 3]>,
}

/// TUN device configuration
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TunConfig<'a> {
    pub enable: bool,
    pub interface: Cow<'a, str>,
    pub mtu: u32,
    pub strict_route: bool,
    pub stack: Cow<'a, str>,
    pub endpoint_independent_nat: bool,
    pub route_address: Vec<Cow<'a, str>>,
    pub route_exclude_address: Vec<Cow<'a, str>>,
    #[serde(default)]
    pub kernel_routing: bool,
    #[serde(default)]
    pub fake_dns: bool,
}

impl<'a> TunConfig<'a> {
    pub fn into_owned(self) -> TunConfig<'static> {
        TunConfig {
            enable: self.enable,
            interface: Cow::Owned(self.interface.into_owned()),
            mtu: self.mtu,
            strict_route: self.strict_route,
            stack: Cow::Owned(self.stack.into_owned()),
            endpoint_independent_nat: self.endpoint_independent_nat,
            route_address: self
                .route_address
                .into_iter()
                .map(|s| Cow::Owned(s.into_owned()))
                .collect(),
            route_exclude_address: self
                .route_exclude_address
                .into_iter()
                .map(|s| Cow::Owned(s.into_owned()))
                .collect(),
            kernel_routing: self.kernel_routing,
            fake_dns: self.fake_dns,
        }
    }
}

/// SOCKS inbound settings
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SocksSettings {
    #[serde(default)]
    pub auth: String, // "password" or "noauth"
    #[serde(default)]
    pub accounts: Vec<Client>,
    #[serde(default)]
    pub udp: bool,
    #[serde(default, rename = "ip")]
    pub bind_address: String,
}

/// Inbound HTTP Account
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct InboundHttpAccount {
    pub user: String,
    pub pass: String,
}

/// Inbound HTTP Settings
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InboundHttpSettings {
    #[serde(default)]
    pub accounts: Vec<InboundHttpAccount>,
    #[serde(default)]
    pub allow_transparent: bool,
}

/// Dokodemo-door settings
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DokodemoSettings {
    pub address: String,
    pub port: u16,
    pub network: String,
    #[serde(default)]
    pub follow_redirect: bool,
}

/// DNS server type
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum DnsServerType {
    Udp,
    Tcp,
    Doh,
}

impl Default for DnsServerType {
    fn default() -> Self {
        Self::Udp
    }
}

/// DNS server configuration
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DnsServerConfig {
    pub address: String,
    pub port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<DnsServerType>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub domains: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expect_ips: Option<Vec<String>>,
}

/// System DNS configuration
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DnsConfig {
    pub enabled: bool,
    pub servers: Vec<DnsServerConfig>,
    #[serde(default)]
    pub disable_cache: bool,
    #[serde(default)]
    pub disable_fallback: bool,
    #[serde(default)]
    pub disable_resolve: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_strategy: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ============================================================================
// Protocol Settings Enum
// ============================================================================

/// Discriminated union for all protocol-specific settings
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "protocol", content = "settings", rename_all = "camelCase")]
pub enum ProtocolSettings<'a> {
    Vless(VlessSettings<'a>),
    Vmess(VmessSettings),
    Trojan(TrojanSettings),
    Shadowsocks(Shadowsocks2022Settings<'a>),
    Socks(SocksSettings),
    Http(InboundHttpSettings),
    Dokodemo(DokodemoSettings),
    Hysteria2(Hysteria2Settings<'a>),
    Tuic(TuicSettings<'a>),
    #[serde(rename = "flowj")]
    FlowJ(FlowJSettings<'a>),
    Naive(NaiveSettings),
    WireGuard(WireGuardSettings),
    Tun(TunConfig<'a>),
}

impl Default for ProtocolSettings<'_> {
    fn default() -> Self {
        ProtocolSettings::Vless(VlessSettings::default())
    }
}

impl ProtocolSettings<'_> {
    /// Get clients from protocols that support them
    pub fn clients(&self) -> Option<&Vec<Client>> {
        match self {
            ProtocolSettings::Vless(s) => Some(&s.clients),
            ProtocolSettings::Vmess(s) => Some(&s.clients),
            ProtocolSettings::Trojan(s) => Some(&s.clients),
            ProtocolSettings::FlowJ(s) => Some(&s.clients),
            _ => None,
        }
    }

    /// Get mutable clients from protocols that support them
    pub fn clients_mut(&mut self) -> Option<&mut Vec<Client>> {
        match self {
            ProtocolSettings::Vless(s) => Some(&mut s.clients),
            ProtocolSettings::Vmess(s) => Some(&mut s.clients),
            ProtocolSettings::Trojan(s) => Some(&mut s.clients),
            ProtocolSettings::FlowJ(s) => Some(&mut s.clients),
            _ => None,
        }
    }

    /// Get the protocol name as a string
    pub fn protocol_name(&self) -> &str {
        match self {
            ProtocolSettings::Vless(_) => "vless",
            ProtocolSettings::Vmess(_) => "vmess",
            ProtocolSettings::Trojan(_) => "trojan",
            ProtocolSettings::Shadowsocks(_) => "shadowsocks",
            ProtocolSettings::Socks(_) => "socks",
            ProtocolSettings::Http(_) => "http",
            ProtocolSettings::Dokodemo(_) => "dokodemo-door",
            ProtocolSettings::Hysteria2(_) => "hysteria2",
            ProtocolSettings::Tuic(_) => "tuic",
            ProtocolSettings::FlowJ(_) => "flowj",
            ProtocolSettings::Naive(_) => "naive",
            ProtocolSettings::WireGuard(_) => "wireguard",
            ProtocolSettings::Tun(_) => "tun",
        }
    }
}

// ============================================================================
// Stream/Transport Settings
// ============================================================================

/// Sniffing configuration
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Sniffing<'a> {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dest_override: Option<Vec<Cow<'a, str>>>,
    #[serde(default, alias = "metadataOnly")]
    pub metadata_only: bool,
    #[serde(default, alias = "routeOnly")]
    pub route_only: bool,
}

/// TLS configuration
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TlsSettings<'a> {
    #[serde(default, skip_serializing_if = "is_cow_empty_str")]
    pub server_name: Cow<'a, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificates: Option<Vec<Certificate<'a>>>,
    #[serde(default, skip_serializing_if = "is_cow_empty_str")]
    pub alpn: Cow<'a, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fragment_settings: Option<FragmentSettings>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "pqcCipher")]
    pub pqc_matrix: Option<PqcMatrix>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// TLS fragment settings for anti-censorship
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FragmentSettings {
    pub enabled: bool,
    pub packets: String,
    pub length: String,
    pub interval: String,
}

/// WebSocket transport settings
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WsSettings<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<serde_json::Value>,
}

/// HTTP/2 transport settings
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HttpSettings<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<Vec<Cow<'a, str>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<Cow<'a, str>>,
    #[serde(default, skip_serializing_if = "is_cow_empty_str")]
    pub method: Cow<'a, str>,
}

/// gRPC transport settings
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GrpcSettings<'a> {
    #[serde(default, skip_serializing_if = "is_cow_empty_str")]
    pub service_name: Cow<'a, str>,
    #[serde(default)]
    pub multi_mode: bool,
}

/// TCP transport settings
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TcpSettings {
    #[serde(default, skip_serializing_if = "String::is_empty", rename = "type")]
    pub header_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<serde_json::Value>,
}

/// KCP transport settings
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct KcpSettings<'a> {
    pub mtu: u32,
    pub tti: u32,
    pub uplink_capacity: u32,
    pub downlink_capacity: u32,
    pub congestion: bool,
    pub read_buffer_size: u32,
    pub write_buffer_size: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<Cow<'a, str>>,
}

/// MQTT transport settings
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MqttTransportSettings<'a> {
    pub broker_address: Cow<'a, str>,
    pub topic: Cow<'a, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(default)]
    pub qos: u8,
}

/// Comprehensive stream settings
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StreamSettings<'a> {
    #[serde(default)]
    pub network: Cow<'a, str>,
    #[serde(default)]
    pub security: Cow<'a, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reality_settings: Option<RealitySettings<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls_settings: Option<TlsSettings<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ws_settings: Option<WsSettings<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_settings: Option<HttpSettings<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kcp_settings: Option<KcpSettings<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grpc_settings: Option<GrpcSettings<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcp_settings: Option<TcpSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mqtt_settings: Option<MqttTransportSettings<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub db_mimic_settings: Option<DbMimicConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slipstream_settings: Option<SlipstreamPlusConfig>,
}

impl<'a> StreamSettings<'a> {
    pub fn total_sni(&self) -> Option<String> {
        if let Some(ref r) = self.reality_settings {
            if let Some(first) = r.server_names.first() {
                return Some(first.to_string());
            }
        }
        if let Some(ref t) = self.tls_settings {
            if !t.server_name.is_empty() {
                return Some(t.server_name.to_string());
            }
        }
        None
    }

    pub fn total_fingerprint(&self) -> Option<String> {
        if let Some(ref r) = self.reality_settings {
            if !r.fingerprint.is_empty() {
                return Some(r.fingerprint.to_string());
            }
        }
        if let Some(ref t) = self.tls_settings {
            if let Some(ref f) = t.fingerprint {
                return Some(f.to_string());
            }
        }
        None
    }

    pub fn total_short_id(&self) -> Option<String> {
        if let Some(ref r) = self.reality_settings {
            if let Some(first) = r.short_ids.first() {
                return Some(first.to_string());
            }
        }
        None
    }

    pub fn total_host(&self) -> Option<String> {
        if let Some(ref w) = self.ws_settings {
            if let Some(headers) = &w.headers {
                if let Some(host) = headers.get("Host") {
                    return host.as_str().map(|s| s.to_string());
                }
            }
        }
        if let Some(ref h) = self.http_settings {
            if let Some(hosts) = &h.host {
                if let Some(first) = hosts.first() {
                    return Some(first.to_string());
                }
            }
        }
        if let Some(ref t) = self.tcp_settings {
            if let Some(req) = &t.request {
                if let Some(headers) = req.get("headers") {
                    if let Some(host) = headers.get("Host") {
                        if let Some(arr) = host.as_array() {
                            if let Some(first) = arr.first() {
                                return first.as_str().map(|s| s.to_string());
                            }
                        } else if let Some(s) = host.as_str() {
                            return Some(s.to_string());
                        }
                    }
                }
            }
        }
        None
    }

    pub fn total_path(&self) -> Option<String> {
        if let Some(ref w) = self.ws_settings {
            if let Some(ref p) = w.path {
                return Some(p.to_string());
            }
        }
        if let Some(ref h) = self.http_settings {
            if let Some(ref p) = h.path {
                return Some(p.to_string());
            }
        }
        if let Some(ref t) = self.tcp_settings {
            if let Some(req) = &t.request {
                if let Some(path) = req.get("path") {
                    if let Some(arr) = path.as_array() {
                        if let Some(first) = arr.first() {
                            return first.as_str().map(|s| s.to_string());
                        }
                    } else if let Some(s) = path.as_str() {
                        return Some(s.to_string());
                    }
                }
            }
        }
        None
    }

    pub fn total_alpn(&self) -> Option<String> {
        if let Some(ref t) = self.tls_settings {
            if !t.alpn.is_empty() {
                return Some(t.alpn.to_string());
            }
        }
        None
    }
}

// ============================================================================
// Inbound/Outbound Models
// ============================================================================

/// Inbound connection configuration
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "server", derive(Validate))]
pub struct Inbound<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "server", serde(alias = "id"))]
    pub id: Option<IdType>,
    /// All-time traffic usage (cumulative, never reset)
    #[serde(default)]
    pub all_time: i64,
    pub remark: Cow<'a, str>,
    pub enable: bool,
    #[serde(alias = "expiryTime")]
    pub expiry: i64,
    /// Traffic reset schedule: "never", "daily", "weekly", "monthly"
    #[serde(default, skip_serializing_if = "is_cow_empty_str")]
    pub traffic_reset: Cow<'a, str>,
    /// Last traffic reset timestamp
    #[serde(default)]
    pub last_traffic_reset_time: i64,
    #[serde(default, alias = "up")]
    pub up_bytes: i64,
    #[serde(default, alias = "down")]
    pub down_bytes: i64,
    #[serde(default)]
    pub total_limit: i64,
    /// Upload speed limit in kbps (0 = unlimited)
    #[serde(default, alias = "upSpeedLimit")]
    pub up_speed_limit: u32,
    /// Download speed limit in kbps (0 = unlimited)
    #[serde(default, alias = "downSpeedLimit")]
    pub down_speed_limit: u32,
    /// Listen address (IP or empty for all interfaces)
    #[serde(default, skip_serializing_if = "is_cow_empty_str")]
    pub listen: Cow<'a, str>,
    #[cfg_attr(feature = "server", validate(range(min = 1, max = 65535)))]
    pub port: u32,
    pub protocol: InboundProtocol,
    pub settings: ProtocolSettings<'a>,
    pub stream_settings: StreamSettings<'a>,
    pub tag: Cow<'a, str>,
    pub sniffing: Sniffing<'a>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Default for Inbound<'_> {
    fn default() -> Self {
        Self {
            id: None,
            all_time: 0,
            remark: Cow::Borrowed(""),
            enable: true,
            expiry: 0,
            traffic_reset: Cow::Borrowed("never"),
            last_traffic_reset_time: 0,
            up_bytes: 0,
            down_bytes: 0,
            total_limit: 0,
            up_speed_limit: 0,
            down_speed_limit: 0,
            listen: Cow::Borrowed(""),
            port: 443,
            protocol: InboundProtocol::Vless,
            settings: ProtocolSettings::default(),
            stream_settings: StreamSettings::default(),
            tag: Cow::Borrowed(""),
            sniffing: Sniffing::default(),
            extra: HashMap::new(),
        }
    }
}

/// Outbound settings for various protocols
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "protocol", content = "settings", rename_all = "camelCase")]
pub enum OutboundSettings {
    Freedom(FreedomSettings),
    Blackhole(BlackholeSettings),
    Vless(VlessSettings<'static>),
    Vmess(VmessSettings),
    Trojan(TrojanSettings),
    Shadowsocks(Shadowsocks2022Settings<'static>),
    Tailscale(TailscaleSettings),
    Tor(TorSettings),
}

impl Default for OutboundSettings {
    fn default() -> Self {
        OutboundSettings::Freedom(FreedomSettings::default())
    }
}

/// Freedom outbound settings (direct connection)
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FreedomSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain_strategy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_level: Option<u32>,
}

/// Blackhole outbound settings (block)
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BlackholeSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<serde_json::Value>,
}

/// Tailscale outbound settings
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TailscaleSettings {
    pub auth_key: String,
    #[serde(default)]
    pub hostname: String,
    #[serde(default)]
    pub ephemeral: bool,
    #[serde(default)]
    pub exit_node: bool,
    #[serde(default)]
    pub accept_routes: bool,
}

/// Tor outbound settings
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TorSettings {
    pub executable_path: String,
    #[serde(default)]
    pub extra_args: Vec<String>,
    #[serde(default)]
    pub data_dir: String,
}

/// Outbound connection configuration
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OutboundModel<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "server", serde(alias = "id"))]
    pub id: Option<IdType>,
    pub remark: Cow<'a, str>,
    pub enable: bool,
    pub protocol: Cow<'a, str>,
    pub settings: OutboundSettings,
    pub stream_settings: StreamSettings<'a>,
    pub tag: Cow<'a, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mux: Option<serde_json::Value>,
}

impl Default for OutboundModel<'_> {
    fn default() -> Self {
        Self {
            id: None,
            remark: Cow::Borrowed(""),
            enable: true,
            protocol: Cow::Borrowed("freedom"),
            settings: OutboundSettings::default(),
            stream_settings: StreamSettings::default(),
            tag: Cow::Borrowed("direct"),
            mux: None,
        }
    }
}

// ============================================================================
// Settings
// ============================================================================

/// Panel/Application settings
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AllSetting {
    pub web_port: u16,
    pub web_cert_file: Option<String>,
    pub web_key_file: Option<String>,
    pub username: String,
    pub password_hash: String,
    pub core_path: Option<String>,
    #[serde(default)]
    pub is_two_factor_enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub two_factor_secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ldap_server_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ldap_base_dn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_api_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub traffic_reset_cron: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warp_license_key: Option<String>,
    #[serde(default = "default_panel_secret_path")]
    pub panel_secret_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decoy_site_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tg_bot_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tg_bot_chat_id: Option<String>,
    #[serde(default)]
    pub tg_bot_enable: bool,
    #[serde(default)]
    pub tg_notify_expiry: bool,
    #[serde(default)]
    pub tg_notify_traffic: bool,
    #[serde(default)]
    pub tg_notify_login: bool,
}

fn default_panel_secret_path() -> String {
    "/panel".to_string()
}

impl Default for AllSetting {
    fn default() -> Self {
        Self {
            web_port: 2053,
            web_cert_file: None,
            web_key_file: None,
            username: "admin".to_string(),
            password_hash: "$argon2id$v=19$m=19456,t=2,p=1$Y2hhbmdlbWUxMjM$2fFf2q5cE5a7a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a".to_string(),
            core_path: None,
            is_two_factor_enabled: false,
            two_factor_secret: None,
            ldap_server_url: None,
            ldap_base_dn: None,
            sub_api_token: None,
            traffic_reset_cron: None,
            warp_license_key: None,
            panel_secret_path: default_panel_secret_path(),
            decoy_site_path: None,
            tg_bot_token: None,
            tg_bot_chat_id: None,
            tg_bot_enable: false,
            tg_notify_expiry: false,
            tg_notify_traffic: false,
            tg_notify_login: false,
        }
    }
}

/// Extended panel settings (compatible with TypeScript PanelSettings)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct PanelSettings {
    pub web_listen: String,
    pub web_domain: String,
    pub web_port: u16,
    pub web_cert_file: String,
    pub web_key_file: String,
    pub web_base_path: String,
    pub session_max_age: u32,
    pub page_size: u32,
    pub expire_diff: i64,
    pub traffic_diff: i64,
    pub remark_model: String,
    pub datepicker: String,
    pub tg_bot_enable: bool,
    pub tg_bot_token: String,
    pub tg_bot_proxy: String,
    #[serde(rename = "tgBotAPIServer")]
    pub tg_bot_api_server: String,
    pub tg_bot_chat_id: String,
    pub tg_run_time: String,
    pub tg_bot_backup: bool,
    pub tg_bot_login_notify: bool,
    pub tg_cpu: u32,
    pub tg_lang: String,
    pub rustray_template_config: String,
    pub secret_enable: bool,
    pub sub_enable: bool,
    pub sub_title: String,
    pub sub_listen: String,
    pub sub_port: u16,
    pub sub_path: String,
    pub sub_domain: String,
    pub sub_cert_file: String,
    pub sub_key_file: String,
    pub sub_updates: u32,
    pub sub_encrypt: bool,
    pub sub_show_info: bool,
    #[serde(rename = "subURI")]
    pub sub_uri: String,
    pub sub_json_path: String,
    #[serde(rename = "subJsonURI")]
    pub sub_json_uri: String,
    pub sub_json_fragment: String,
    pub sub_json_noises: String,
    pub sub_json_mux: String,
    pub sub_json_rules: String,
    pub time_location: String,
}

// ============================================================================
// Subscription Group
// ============================================================================

/// Subscription group for managing multiple inbounds
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionGroup {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "server", serde(alias = "id"))]
    pub id: Option<IdType>,
    pub remark: String,
    pub inbound_ids: Vec<String>,
    pub is_enabled: bool,
    pub expiry_time: i64,
    #[serde(default)]
    pub total_traffic: i64,
    #[serde(default)]
    pub used_traffic: i64,
    #[serde(default)]
    pub created_at: i64,
}

impl Default for SubscriptionGroup {
    fn default() -> Self {
        Self {
            id: None,
            remark: String::new(),
            inbound_ids: Vec::new(),
            is_enabled: true,
            expiry_time: 0,
            total_traffic: 0,
            used_traffic: 0,
            created_at: chrono::Utc::now().timestamp(),
        }
    }
}

// ============================================================================
// UI-Specific Types
// ============================================================================

/// Theme options
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Theme {
    #[default]
    Light,
    Dark,
    UltraDark,
}

/// Locale/language options
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub enum Locale {
    #[default]
    #[serde(rename = "en_US")]
    EnUs,
    #[serde(rename = "zh_CN")]
    ZhCn,
    #[serde(rename = "zh_TW")]
    ZhTw,
    #[serde(rename = "fa_IR")]
    FaIr,
    #[serde(rename = "ru_RU")]
    RuRu,
    #[serde(rename = "ar_EG")]
    ArEg,
    #[serde(rename = "es_ES")]
    EsEs,
    #[serde(rename = "id_ID")]
    IdId,
    #[serde(rename = "ja_JP")]
    JaJp,
    #[serde(rename = "pt_BR")]
    PtBr,
    #[serde(rename = "tr_TR")]
    TrTr,
    #[serde(rename = "uk_UA")]
    UkUa,
    #[serde(rename = "vi_VN")]
    ViVn,
}

// ============================================================================
// Response Wrapper Types (for API compatibility)
// ============================================================================

/// General response for non-typed responses
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GeneralResponse {
    pub success: bool,
    pub msg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub obj: Option<serde_json::Value>,
}

impl GeneralResponse {
    pub fn success(msg: &str, obj: Option<serde_json::Value>) -> Self {
        Self {
            success: true,
            msg: msg.to_string(),
            obj,
        }
    }

    pub fn error(msg: &str) -> Self {
        Self {
            success: false,
            msg: msg.to_string(),
            obj: None,
        }
    }
}

// Add TrafficStats
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct TrafficStats {
    pub name: String,
    pub value: i64,
}

// Ensure InboundListResponse uses Inbound
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InboundListResponse<'a> {
    pub inbounds: Vec<Inbound<'a>>,
}

// ============================================================================
// Audit Event Types (Security Audit & Log Intelligence)
// ============================================================================

/// Action types for audit logging
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    #[default]
    Unknown,
    // Authentication
    Login,
    LoginFailed,
    Logout,
    PasswordChanged,
    TwoFactorEnabled,
    TwoFactorDisabled,
    // Inbound Management
    InboundCreated,
    InboundUpdated,
    InboundDeleted,
    InboundEnabled,
    InboundDisabled,
    // Client Management
    ClientCreated,
    ClientUpdated,
    ClientDeleted,
    ClientEnabled,
    ClientDisabled,
    ClientTrafficReset,
    // System Configuration
    SettingsUpdated,
    CoreStarted,
    CoreStopped,
    CoreRestarted,
    GeoFilesUpdated,
    CertificateUpdated,
    // Security Events
    IpBanned,
    IpUnbanned,
    RateLimited,
    SuspiciousActivity,
}

/// Audit event record for security logging
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AuditEvent {
    /// Unique identifier
    #[cfg(feature = "server")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<IdType>,
    #[cfg(not(feature = "server"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Unix timestamp (milliseconds)
    pub timestamp: i64,

    /// Action that was performed
    pub action: AuditAction,

    /// User who performed the action (if authenticated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    /// IP address of the client
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,

    /// User agent string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,

    /// Additional details about the action (JSON)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,

    /// Whether the action was successful
    pub success: bool,

    /// Error message if action failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Default for AuditEvent {
    fn default() -> Self {
        Self {
            id: None,
            timestamp: chrono::Utc::now().timestamp_millis(),
            action: AuditAction::Unknown,
            user: None,
            ip_address: None,
            user_agent: None,
            details: None,
            success: true,
            error: None,
        }
    }
}

impl AuditEvent {
    /// Create a new successful audit event
    pub fn new(action: AuditAction) -> Self {
        Self {
            action,
            ..Default::default()
        }
    }

    /// Create a failed audit event with error message
    pub fn failed(action: AuditAction, error: impl Into<String>) -> Self {
        Self {
            action,
            success: false,
            error: Some(error.into()),
            ..Default::default()
        }
    }

    /// Set the user who performed the action
    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Set the IP address
    pub fn with_ip(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    /// Set the user agent
    pub fn with_user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = Some(ua.into());
        self
    }

    /// Set additional details
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

// ============================================================================
// Mesh Node Types (Multi-Node Orchestration)
// ============================================================================

/// Status of a mesh node
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum MeshNodeStatus {
    #[default]
    Unknown,
    Online,
    Offline,
    Syncing,
    Degraded,
    Maintenance,
}

/// Role of a mesh node in the cluster
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum MeshNodeRole {
    #[default]
    Standalone,
    Primary,
    Secondary,
    Observer,
}

/// Mesh node for multi-node orchestration
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MeshNode {
    /// Unique node identifier
    #[cfg(feature = "server")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<IdType>,
    #[cfg(not(feature = "server"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Explicit Node ID (may duplicate id but useful for consistency)
    pub node_id: String,

    /// Human-readable node name
    pub name: String,

    /// Region/Zone
    pub region: String,

    /// Node address (hostname/IP)
    pub address: String,

    /// Port
    pub port: u16,

    /// API endpoint URL
    pub api_url: String,

    /// Node role in the cluster
    pub role: MeshNodeRole,

    /// Current node status
    pub status: MeshNodeStatus,

    /// Health metrics
    pub health: NodeHealth,

    /// Capacity metrics
    pub capacity: NodeCapacity,

    /// Last heartbeat timestamp (milliseconds)
    pub last_heartbeat: i64,

    /// Node version string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Number of active clients on this node
    pub client_count: u32,

    /// Whether this node is the local/self node
    pub is_local: bool,

    /// API token for authentication  
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_token: Option<String>,

    /// Registration timestamp
    pub created_at: i64,
}

impl Default for MeshNode {
    fn default() -> Self {
        Self {
            id: None,
            node_id: "unknown".to_string(),
            name: "node".to_string(),
            region: "unknown".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8080,
            api_url: "http://127.0.0.1:8080/api".to_string(),
            role: MeshNodeRole::Standalone,
            status: MeshNodeStatus::Unknown,
            health: NodeHealth::default(),
            capacity: NodeCapacity::default(),
            last_heartbeat: 0,
            version: None,
            client_count: 0,
            is_local: false,
            api_token: None,
            created_at: chrono::Utc::now().timestamp_millis(),
        }
    }
}

impl MeshNode {
    /// Create a new mesh node with name and address
    pub fn new(name: impl Into<String>, address: impl Into<String>, port: u16) -> Self {
        let name = name.into();
        let address = address.into();
        let api_url = format!("http://{}:{}/api", address, port);

        Self {
            name,
            address,
            port,
            api_url,
            ..Default::default()
        }
    }

    /// Create a local node representing this server
    pub fn local(name: impl Into<String>, port: u16) -> Self {
        let name = name.into();
        let address = "127.0.0.1".to_string();
        let api_url = format!("http://127.0.0.1:{}/api", port);

        Self {
            name,
            address,
            port,
            api_url,
            is_local: true,
            status: MeshNodeStatus::Online,
            last_heartbeat: chrono::Utc::now().timestamp_millis(),
            ..Default::default()
        }
    }

    /// Update the heartbeat timestamp
    pub fn heartbeat(&mut self) {
        self.last_heartbeat = chrono::Utc::now().timestamp_millis();
        self.status = MeshNodeStatus::Online;
    }

    /// Check if node is considered stale (no heartbeat in 60 seconds)
    pub fn is_stale(&self) -> bool {
        let now = chrono::Utc::now().timestamp_millis();
        now - self.last_heartbeat > 60_000
    }
}

/// Cluster statistics summary
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct ClusterStats {
    pub total_nodes: usize,
    pub online_nodes: usize,
    pub offline_nodes: usize,
    pub total_clients: u32,
}
// ============================================================================
// Speed Test Models
// ============================================================================

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct SpeedTestResults {
    pub server: String,
    pub latency_ms: f64,
    pub jitter_ms: f64,
    pub download_mbps: f64,
    pub upload_mbps: f64, // Placeholder if we add upload test later
    pub packet_loss: f64, // Placeholder
    pub ttfb_ms: f64,
    pub timestamp: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub network: Option<String>,
    pub security: Option<String>,
    pub sni: Option<String>,
    pub pbk: Option<String>,
    pub sid: Option<String>,
    pub path: Option<String>,
    pub fingerprint: Option<String>,
    pub allow_insecure: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ScannerType {
    Dns,
    Cloudflare,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub ip: String,
    pub status: String,
    pub latency_ms: f64,
    pub resolver_type: Option<String>,
}

// ============================================================================
// NOC Dashboard Telemetry Models
// ============================================================================

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct NodeHealth {
    pub cpu_percent: f32,
    pub memory_percent: f32,
    pub disk_percent: f32,
    pub latency_ms: f32,
    pub packet_loss_percent: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct NodeCapacity {
    pub max_users: u32,
    pub current_users: u32,
    pub max_bandwidth_mbps: u32,
    pub current_bandwidth_mbps: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct ActiveConnection {
    pub id: String,
    pub local_ip: String,
    pub remote_ip: String,
    pub protocol: String,
    pub transport: String,
    pub rtt_ms: f64,
    pub handshake_ttfb_ms: f64,
    pub jitter_ms: f64,
    pub upload_bytes: u64,
    pub download_bytes: u64,
    pub started_at: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryState {
    #[default]
    Idle,
    Scanning,
    ReVerifying,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct DashboardStats {
    pub active_connections: Vec<ActiveConnection>,
    pub discovery_state: DiscoveryState,
    pub mesh_stats: ClusterStats,
    pub node_health: Option<NodeHealth>,
}

impl NodeCapacity {
    pub fn utilization_percent(&self) -> f32 {
        let user_util = (self.current_users as f32 / self.max_users.max(1) as f32) * 100.0;
        let bw_util =
            (self.current_bandwidth_mbps as f32 / self.max_bandwidth_mbps.max(1) as f32) * 100.0;
        user_util.max(bw_util)
    }

    pub fn available_slots(&self) -> u32 {
        self.max_users.saturating_sub(self.current_users)
    }
}

impl NodeHealth {
    pub fn is_healthy(&self) -> bool {
        self.cpu_percent < 80.0
            && self.memory_percent < 85.0
            && self.disk_percent < 90.0
            && self.latency_ms < 200.0
            && self.packet_loss_percent < 1.0
    }

    pub fn health_score(&self) -> f32 {
        let cpu_score = (100.0 - self.cpu_percent) / 100.0;
        let mem_score = (100.0 - self.memory_percent) / 100.0;
        let disk_score = (100.0 - self.disk_percent) / 100.0;
        let latency_score = 1.0 - (self.latency_ms / 500.0).min(1.0);
        let loss_score = 1.0 - (self.packet_loss_percent / 5.0).min(1.0);

        (cpu_score + mem_score + disk_score + latency_score + loss_score) / 5.0
    }
}

// ============================================================================
// Advanced Transport Configurations
// ============================================================================

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PortType {
    #[default]
    StaticRange,
    RandomDynamic,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CongestionControl {
    #[default]
    Cubic,
    BBR,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct FlowJConfig {
    pub port_count: u8, // 1-64
    pub port_type: PortType,
    pub congestion_control: CongestionControl,
    pub padding_strategy: u8,
    pub jitter_ms: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum MimicTarget {
    #[default]
    PostgreSQL,
    Redis,
    MySQL,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct DbMimicConfig {
    pub target: MimicTarget,
    pub fake_db_name: String,
    pub fake_user: String,
    pub startup_payload_hex: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum DnsRecordType {
    #[default]
    TXT,
    A,
    AAAA,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct SlipstreamPlusConfig {
    pub root_domain: String,
    pub record_type: DnsRecordType,
    pub udp_frag_limit: u16,
}

// ============================================================================
// Scanner & Intelligence Models
// ============================================================================

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct ScannerConfig {
    pub concurrency: u32,
    pub timeout_ms: u32,
    pub cidr_ranges: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct CleanPath {
    pub ip: String,
    pub isp: String,
    pub score: u8,
    pub found_at: i64,
    pub last_checked: i64,
    pub status: String, // "Active", "Aging", "Dead"
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct DnsResolverStatus {
    pub resolver_ip: String,
    pub is_poisoned: bool,
    pub latency_ms: u32,
    pub query_hash: String,
}

// ============================================================================
// System Lifecycle Models
// ============================================================================

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct SystemHealth {
    pub open_sockets: u32,
    pub thread_count: u32,
    pub uptime_seconds: u64,
    pub memory_usage_mb: u64,
    pub core_status: String, // "Running", "Stopped", "Starting"
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct AssetStatus {
    pub name: String,
    pub version: String,
    pub hash: String,
    pub last_updated: i64,
    pub file_size_bytes: u64,
}
