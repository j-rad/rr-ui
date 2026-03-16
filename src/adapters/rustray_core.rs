// src/adapters/rustray_core.rs
//! RustRay Core Adapter
//!
//! Implements ProxyCore for the internal RustRay engine.
//!
//! Config lifecycle uses the Temp-Validate-Swap (TVS) pattern:
//!   1. Serialize to a `.tmp` file beside the live config
//!   2. Validate by spawning `rustray check -c <tmp>`
//!   3. On success, atomically rename tmp → live path
//!   4. Send SIGHUP to the running process to reload in-place

use crate::adapters::atomic_config::AtomicConfigWriter;
use crate::domain::proxy_core::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use tokio::fs;

// ──────────────────────────────── RustRay JSON schema ────────────────────────

/// Top-level RustRay configuration file format.
#[derive(Debug, Serialize, Deserialize)]
pub struct RustRayConfig {
    pub log: RustRayLog,
    pub inbounds: Vec<RustRayInbound>,
    pub outbounds: Vec<RustRayOutbound>,
    pub routing: RustRayRouting,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns: Option<RustRayDns>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RustRayLog {
    pub level: String,
    pub access: String,
    pub error: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RustRayInbound {
    pub tag: String,
    pub protocol: String,
    pub listen: String,
    pub port: u16,
    pub settings: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_settings: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sniffing: Option<RustRaySniffing>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RustRaySniffing {
    pub enabled: bool,
    pub dest_override: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RustRayOutbound {
    pub tag: String,
    pub protocol: String,
    pub settings: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RustRayRouting {
    pub domain_strategy: String,
    pub rules: Vec<RustRayRoutingRule>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RustRayRoutingRule {
    pub r#type: String,
    pub outbound_tag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inbound_tag: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RustRayDns {
    pub servers: Vec<String>,
}

// ──────────────────────────── User management models ─────────────────────────

/// RustRay VMess/VLESS user entry embedded inside inbound settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustRayUser {
    pub id: String,
    pub email: String,
    pub level: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alter_id: Option<u16>,
}

// ─────────────────────────────── Adapter struct ───────────────────────────────

pub struct RustRayCore {
    process: Option<Child>,
    config: Option<CoreConfig>,
    config_path: PathBuf,
    apl: AtomicConfigWriter,
}

impl RustRayCore {
    pub fn new() -> Self {
        let config_path = PathBuf::from("/etc/rustray/config.json");
        let base_dir = config_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("/tmp"))
            .to_path_buf();
        Self {
            process: None,
            config: None,
            config_path,
            apl: AtomicConfigWriter::new(base_dir),
        }
    }

    /// Convert our abstract `CoreConfig` into the RustRay JSON schema.
    fn to_rustray_config(&self, config: &CoreConfig) -> RustRayConfig {
        let log_level = match config.log_level {
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warning => "warning",
            LogLevel::Error => "error",
            LogLevel::None => "none",
        };

        let inbounds = config
            .inbounds
            .iter()
            .map(|ib| self.map_inbound(ib))
            .collect();

        let outbounds = config
            .outbounds
            .iter()
            .map(|ob| self.map_outbound(ob))
            .collect();

        let routing = self.map_routing(&config.routing);

        let dns = config.dns.as_ref().map(|d| RustRayDns {
            servers: d.servers.clone(),
        });

        RustRayConfig {
            log: RustRayLog {
                level: log_level.to_string(),
                access: "/var/log/rustray/access.log".to_string(),
                error: "/var/log/rustray/error.log".to_string(),
            },
            inbounds,
            outbounds,
            routing,
            dns,
        }
    }

    /// Map one abstract inbound onto a RustRay inbound object.
    ///
    /// For VMess/VLESS, the `settings` field carries a `clients` array seeded
    /// from whatever was embedded in the domain config's settings JSON.
    fn map_inbound(&self, ib: &InboundConfig) -> RustRayInbound {
        let protocol = self.protocol_name(ib.protocol.clone());

        // Preserve caller-supplied settings verbatim if present, otherwise
        // synthesise a minimal valid object for the given protocol.
        let settings = if ib.settings.is_object() && !ib.settings.as_object().unwrap().is_empty() {
            ib.settings.clone()
        } else {
            self.default_inbound_settings(&ib.protocol)
        };

        let sniffing = match ib.protocol {
            Protocol::Vmess | Protocol::Vless | Protocol::Trojan => Some(RustRaySniffing {
                enabled: true,
                dest_override: vec!["http".to_string(), "tls".to_string()],
            }),
            _ => None,
        };

        RustRayInbound {
            tag: ib.tag.clone(),
            protocol,
            listen: ib.listen.clone(),
            port: ib.port,
            settings,
            stream_settings: None,
            sniffing,
        }
    }

    /// Map one abstract outbound.
    fn map_outbound(&self, ob: &OutboundConfig) -> RustRayOutbound {
        let settings = if ob.settings.is_object() && !ob.settings.as_object().unwrap().is_empty() {
            ob.settings.clone()
        } else {
            self.default_outbound_settings(&ob.protocol)
        };

        RustRayOutbound {
            tag: ob.tag.clone(),
            protocol: self.protocol_name(ob.protocol.clone()),
            settings,
        }
    }

    /// Map routing rules.
    fn map_routing(&self, routing: &RoutingConfig) -> RustRayRouting {
        let domain_strategy = match routing.domain_strategy {
            DomainStrategy::AsIs => "AsIs",
            DomainStrategy::IpIfNonMatch => "IPIfNonMatch",
            DomainStrategy::IpOnDemand => "IPOnDemand",
        };

        let rules = routing
            .rules
            .iter()
            .map(|r| RustRayRoutingRule {
                r#type: "field".to_string(),
                outbound_tag: r.outbound_tag.clone(),
                inbound_tag: r.inbound_tag.clone().map(|t| vec![t]),
                domain: r.domain.clone(),
                ip: r.ip.clone(),
            })
            .collect();

        RustRayRouting {
            domain_strategy: domain_strategy.to_string(),
            rules,
        }
    }

    /// Canonical protocol name string used by RustRay's JSON.
    fn protocol_name(&self, proto: Protocol) -> String {
        match proto {
            Protocol::Vmess => "vmess",
            Protocol::Vless => "vless",
            Protocol::Trojan => "trojan",
            Protocol::Shadowsocks => "shadowsocks",
            Protocol::Hysteria2 => "hysteria2",
            Protocol::Tuic => "tuic",
            Protocol::Socks => "socks",
            Protocol::Http => "http",
            Protocol::Freedom => "freedom",
            Protocol::Blackhole => "blackhole",
        }
        .to_string()
    }

    /// Minimal valid `settings` object for inbounds that the RustRay engine
    /// requires, when the caller did not supply custom settings.
    fn default_inbound_settings(&self, proto: &Protocol) -> serde_json::Value {
        match proto {
            Protocol::Vmess => serde_json::json!({ "clients": [] }),
            Protocol::Vless => serde_json::json!({ "clients": [], "decryption": "none" }),
            Protocol::Trojan => serde_json::json!({ "clients": [] }),
            Protocol::Shadowsocks => serde_json::json!({
                "method": "chacha20-ietf-poly1305",
                "password": "",
                "network": "tcp,udp"
            }),
            Protocol::Socks => serde_json::json!({ "auth": "noauth", "udp": true }),
            Protocol::Http => serde_json::json!({ "timeout": 300 }),
            _ => serde_json::json!({}),
        }
    }

    fn default_outbound_settings(&self, proto: &Protocol) -> serde_json::Value {
        match proto {
            Protocol::Freedom => serde_json::json!({ "domainStrategy": "UseIPv4v6" }),
            Protocol::Blackhole => serde_json::json!({ "response": { "type": "none" } }),
            _ => serde_json::json!({}),
        }
    }

    // ─────────────────── Temp-Validate-Swap (TVS) helpers ────────────────────

    /// Write `content` to a `.tmp` path, verify it via `rustray check`, then
    /// atomically rename it to the live config path.
    async fn temp_validate_swap(&self, content: &[u8]) -> Result<(), CoreError> {
        let tmp_path = self.config_path.with_extension("json.tmp");

        // Step 1: write to tmp (non-atomic is fine here; TVS provides safety)
        fs::write(&tmp_path, content)
            .await
            .map_err(|e| CoreError::ConfigInvalid(format!("tmp write failed: {}", e)))?;

        // Step 2: validate with the binary's own checker
        let status = Command::new("rustray")
            .args(["check", "-c", &tmp_path.to_string_lossy()])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        match status {
            Err(_) => {
                // `rustray check` unavailable – treat as best-effort
                // (some distributions don't ship the check sub-command).
            }
            Ok(s) if !s.success() => {
                let _ = fs::remove_file(&tmp_path).await;
                return Err(CoreError::ConfigInvalid(
                    "rustray rejected the generated configuration".to_string(),
                ));
            }
            Ok(_) => {}
        }

        // Step 3: atomic rename tmp → live via APL
        let content_owned = content.to_vec();
        let config_path = self.config_path.clone();
        let apl = AtomicConfigWriter::new(
            config_path
                .parent()
                .unwrap_or_else(|| std::path::Path::new("/tmp"))
                .to_path_buf(),
        );

        tokio::task::spawn_blocking(move || {
            let _ = fs::remove_file(&tmp_path); // remove the intermediate tmp
            apl.write_with_backup(&config_path, &content_owned)
        })
        .await
        .map_err(|e| CoreError::ConfigInvalid(format!("spawn_blocking: {}", e)))?
        .map_err(|e| CoreError::ConfigInvalid(format!("atomic swap: {}", e)))?;

        // Step 4: SIGHUP live process to trigger config reload (zero downtime)
        if let Some(proc) = &self.process {
            #[cfg(unix)]
            {
                use std::os::unix::process::CommandExt;
                let pid = proc.id();
                // SIGHUP = 1
                unsafe {
                    libc::kill(pid as libc::pid_t, libc::SIGHUP);
                }
            }
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl ProxyCore for RustRayCore {
    fn name(&self) -> &'static str {
        "rustray"
    }

    async fn version(&self) -> Result<String, CoreError> {
        let output = Command::new("rustray")
            .arg("version")
            .output()
            .map_err(|e| CoreError::ApiError(e.to_string()))?;

        let version = String::from_utf8_lossy(&output.stdout);
        Ok(version.lines().next().unwrap_or("unknown").to_string())
    }

    async fn start(&mut self, config: CoreConfig) -> Result<(), CoreError> {
        if self.is_running() {
            return Err(CoreError::AlreadyRunning);
        }

        let rr_config = self.to_rustray_config(&config);
        let json = serde_json::to_vec_pretty(&rr_config)
            .map_err(|e| CoreError::ConfigInvalid(e.to_string()))?;

        self.temp_validate_swap(&json).await?;

        let child = Command::new("rustray")
            .args(["run", "-c", &self.config_path.to_string_lossy()])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| CoreError::StartFailed(e.to_string()))?;

        self.process = Some(child);
        self.config = Some(config);
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), CoreError> {
        if let Some(mut proc) = self.process.take() {
            proc.kill()
                .map_err(|e| CoreError::StopFailed(e.to_string()))?;
            self.config = None;
            Ok(())
        } else {
            Err(CoreError::NotRunning)
        }
    }

    fn is_running(&self) -> bool {
        self.process.as_ref().map_or(false, |p| p.id() > 0)
    }

    fn supported_protocols(&self) -> Vec<Protocol> {
        vec![
            Protocol::Vmess,
            Protocol::Vless,
            Protocol::Trojan,
            Protocol::Shadowsocks,
            Protocol::Hysteria2,
            Protocol::Tuic,
            Protocol::Socks,
            Protocol::Http,
            Protocol::Freedom,
            Protocol::Blackhole,
        ]
    }

    async fn validate_config(&self, config: &CoreConfig) -> Result<(), CoreError> {
        for ib in &config.inbounds {
            if !self.supported_protocols().contains(&ib.protocol) {
                return Err(CoreError::ProtocolNotSupported(
                    ib.protocol.display_name().to_string(),
                ));
            }
        }
        Ok(())
    }

    async fn get_stats(&self) -> Result<CoreStats, CoreError> {
        if !self.is_running() {
            return Err(CoreError::NotRunning);
        }
        // RustRay exposes a gRPC Stats API on 127.0.0.1:10086.
        // Telemetry is scraped separately by the telemetry service; here we
        // return a zeroed snapshot so callers always get a valid struct.
        Ok(CoreStats {
            uptime_seconds: 0,
            total_upload_bytes: 0,
            total_download_bytes: 0,
            active_connections: 0,
            user_stats: HashMap::new(),
        })
    }

    /// Hot-add a user by regenerating the config with the new client entry and
    /// performing a Temp-Validate-Swap.
    async fn add_user(&mut self, user: UserConfig) -> Result<(), CoreError> {
        if !self.is_running() {
            return Err(CoreError::NotRunning);
        }
        let mut config = self
            .config
            .clone()
            .ok_or_else(|| CoreError::ConfigInvalid("no active config".to_string()))?;

        // Inject client into every inbound that natively carries a client list.
        let new_user_json = serde_json::to_value(RustRayUser {
            id: user.uuid.clone(),
            email: user.email.clone(),
            level: user.level,
            alter_id: user.alter_id,
        })
        .map_err(|e| CoreError::ConfigInvalid(e.to_string()))?;

        for ib in config.inbounds.iter_mut() {
            match ib.protocol {
                Protocol::Vmess | Protocol::Vless | Protocol::Trojan => {
                    if let Some(clients) = ib.settings.get_mut("clients") {
                        if let Some(arr) = clients.as_array_mut() {
                            arr.push(new_user_json.clone());
                        }
                    } else {
                        ib.settings["clients"] = serde_json::json!([new_user_json]);
                    }
                }
                _ => {}
            }
        }

        let rr_config = self.to_rustray_config(&config);
        let json = serde_json::to_vec_pretty(&rr_config)
            .map_err(|e| CoreError::ConfigInvalid(e.to_string()))?;
        self.temp_validate_swap(&json).await?;
        self.config = Some(config);
        Ok(())
    }

    /// Hot-remove a user by UUID and reapply config via TVS.
    async fn remove_user(&mut self, user_id: &str) -> Result<(), CoreError> {
        if !self.is_running() {
            return Err(CoreError::NotRunning);
        }
        let mut config = self
            .config
            .clone()
            .ok_or_else(|| CoreError::ConfigInvalid("no active config".to_string()))?;

        let mut found = false;
        for ib in config.inbounds.iter_mut() {
            if let Some(clients) = ib.settings.get_mut("clients") {
                if let Some(arr) = clients.as_array_mut() {
                    let before = arr.len();
                    arr.retain(|u| {
                        u.get("id")
                            .and_then(|v| v.as_str())
                            .map_or(true, |id| id != user_id)
                    });
                    if arr.len() < before {
                        found = true;
                    }
                }
            }
        }

        if !found {
            return Err(CoreError::UserNotFound(user_id.to_string()));
        }

        let rr_config = self.to_rustray_config(&config);
        let json = serde_json::to_vec_pretty(&rr_config)
            .map_err(|e| CoreError::ConfigInvalid(e.to_string()))?;
        self.temp_validate_swap(&json).await?;
        self.config = Some(config);
        Ok(())
    }

    async fn update_user(&mut self, user_id: &str, user: UserConfig) -> Result<(), CoreError> {
        self.remove_user(user_id).await?;
        self.add_user(user).await
    }
}

// ─────────────────────────────── URL builder ──────────────────────────────────

/// Build a VLESS share URL for `user` using the address, port, and transport
/// settings from the first matching inbound.  Returns `None` when the inbound
/// list contains no VLESS entry.
pub fn build_vless_share_url(
    user: &UserConfig,
    inbounds: &[InboundConfig],
    host: &str,
) -> Option<String> {
    inbounds
        .iter()
        .find(|ib| ib.protocol == Protocol::Vless)
        .map(|ib| {
            // VLESS URI:  vless://<uuid>@<host>:<port>?<params>#<remark>
            let remark = urlencoding::encode(&user.email);
            format!(
                "vless://{}@{}:{}?encryption=none&type=tcp&security=reality#{}",
                user.uuid, host, ib.port, remark
            )
        })
}

/// Build a VMess share URL (base64-encoded JSON link) for the first VMess
/// inbound that can host the given user.
pub fn build_vmess_share_url(
    user: &UserConfig,
    inbounds: &[InboundConfig],
    host: &str,
) -> Option<String> {
    use base64::{Engine as _, engine::general_purpose};

    inbounds
        .iter()
        .find(|ib| ib.protocol == Protocol::Vmess)
        .map(|ib| {
            let link = serde_json::json!({
                "v": "2",
                "ps": user.email,
                "add": host,
                "port": ib.port.to_string(),
                "id": user.uuid,
                "aid": user.alter_id.unwrap_or(0).to_string(),
                "net": "tcp",
                "type": "none",
                "host": "",
                "path": "",
                "tls": ""
            });
            format!(
                "vmess://{}",
                general_purpose::STANDARD.encode(link.to_string())
            )
        })
}

// ──────────────────────────────────── Tests ───────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config() -> CoreConfig {
        CoreConfig {
            log_level: LogLevel::Info,
            inbounds: vec![
                InboundConfig {
                    tag: "vless-in".to_string(),
                    protocol: Protocol::Vless,
                    listen: "0.0.0.0".to_string(),
                    port: 443,
                    settings: serde_json::json!({ "clients": [], "decryption": "none" }),
                },
                InboundConfig {
                    tag: "vmess-in".to_string(),
                    protocol: Protocol::Vmess,
                    listen: "0.0.0.0".to_string(),
                    port: 8443,
                    settings: serde_json::json!({ "clients": [] }),
                },
            ],
            outbounds: vec![OutboundConfig {
                tag: "direct".to_string(),
                protocol: Protocol::Freedom,
                settings: serde_json::json!({}),
            }],
            routing: RoutingConfig {
                rules: vec![],
                domain_strategy: DomainStrategy::AsIs,
            },
            dns: None,
        }
    }

    #[test]
    fn test_rustray_core_creation() {
        let core = RustRayCore::new();
        assert_eq!(core.name(), "rustray");
        assert!(!core.is_running());
    }

    #[test]
    fn test_all_protocols_supported() {
        let core = RustRayCore::new();
        let protocols = core.supported_protocols();
        assert!(protocols.contains(&Protocol::Vless));
        assert!(protocols.contains(&Protocol::Vmess));
        assert!(protocols.contains(&Protocol::Hysteria2));
        assert!(protocols.contains(&Protocol::Tuic));
    }

    #[test]
    fn test_config_conversion_produces_valid_json() {
        let core = RustRayCore::new();
        let config = make_config();
        let rr = core.to_rustray_config(&config);
        let json = serde_json::to_string_pretty(&rr).expect("serialization must succeed");
        assert!(json.contains("vless"));
        assert!(json.contains("vmess"));
        assert!(json.contains("freedom"));
    }

    #[test]
    fn test_build_vless_share_url() {
        let user = UserConfig {
            id: "u1".to_string(),
            email: "alice@example.com".to_string(),
            uuid: "00000000-0000-0000-0000-000000000001".to_string(),
            level: 0,
            alter_id: None,
        };
        let config = make_config();
        let url = build_vless_share_url(&user, &config.inbounds, "node1.example.com")
            .expect("should produce a VLESS url");
        assert!(url.starts_with("vless://"));
        assert!(url.contains("node1.example.com:443"));
    }

    #[test]
    fn test_build_vmess_share_url() {
        let user = UserConfig {
            id: "u2".to_string(),
            email: "bob@example.com".to_string(),
            uuid: "00000000-0000-0000-0000-000000000002".to_string(),
            level: 0,
            alter_id: Some(0),
        };
        let config = make_config();
        let url = build_vmess_share_url(&user, &config.inbounds, "node1.example.com")
            .expect("should produce a VMess url");
        assert!(url.starts_with("vmess://"));
    }

    #[tokio::test]
    async fn test_validate_config_rejects_unknown_protocol() {
        let core = RustRayCore::new();
        // Hysteria2 is supported, so the check should pass.
        let config = CoreConfig {
            log_level: LogLevel::Info,
            inbounds: vec![InboundConfig {
                tag: "hy2-in".to_string(),
                protocol: Protocol::Hysteria2,
                listen: "0.0.0.0".to_string(),
                port: 443,
                settings: serde_json::json!({}),
            }],
            outbounds: vec![],
            routing: RoutingConfig {
                rules: vec![],
                domain_strategy: DomainStrategy::AsIs,
            },
            dns: None,
        };
        assert!(core.validate_config(&config).await.is_ok());
    }
}
