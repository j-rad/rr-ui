// src/adapters/singbox_core.rs
//! Sing-box Core Adapter
//!
//! Implements ProxyCore trait for Sing-box (Hysteria2, TUIC support)

use crate::domain::proxy_core::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use tokio::fs;

/// Sing-box core implementation
pub struct SingboxCore {
    process: Option<Child>,
    config: Option<CoreConfig>,
    config_path: String,
    api_address: String,
}

impl SingboxCore {
    pub fn new() -> Self {
        Self {
            process: None,
            config: None,
            config_path: "/tmp/singbox_config.json".to_string(),
            api_address: "127.0.0.1:9090".to_string(),
        }
    }

    /// Convert CoreConfig to Sing-box format
    fn convert_config(&self, config: &CoreConfig) -> SingboxConfig {
        SingboxConfig {
            log: SingboxLog {
                level: match config.log_level {
                    LogLevel::Debug => "debug",
                    LogLevel::Info => "info",
                    LogLevel::Warning => "warn",
                    LogLevel::Error => "error",
                    LogLevel::None => "panic",
                }
                .to_string(),
            },
            inbounds: config
                .inbounds
                .iter()
                .map(|ib| self.convert_inbound(ib))
                .collect(),
            outbounds: config
                .outbounds
                .iter()
                .map(|ob| self.convert_outbound(ob))
                .collect(),
            route: self.convert_routing(&config.routing),
        }
    }

    fn convert_inbound(&self, inbound: &InboundConfig) -> SingboxInbound {
        SingboxInbound {
            tag: inbound.tag.clone(),
            r#type: match inbound.protocol {
                Protocol::Vmess => "vmess",
                Protocol::Vless => "vless",
                Protocol::Trojan => "trojan",
                Protocol::Shadowsocks => "shadowsocks",
                Protocol::Hysteria2 => "hysteria2",
                Protocol::Tuic => "tuic",
                Protocol::Socks => "socks",
                Protocol::Http => "http",
                _ => "mixed",
            }
            .to_string(),
            listen: inbound.listen.clone(),
            listen_port: inbound.port,
            users: Vec::new(),
        }
    }

    fn convert_outbound(&self, outbound: &OutboundConfig) -> SingboxOutbound {
        SingboxOutbound {
            tag: outbound.tag.clone(),
            r#type: match outbound.protocol {
                Protocol::Freedom => "direct",
                Protocol::Blackhole => "block",
                Protocol::Vmess => "vmess",
                Protocol::Vless => "vless",
                Protocol::Trojan => "trojan",
                Protocol::Shadowsocks => "shadowsocks",
                Protocol::Hysteria2 => "hysteria2",
                Protocol::Tuic => "tuic",
                _ => "direct",
            }
            .to_string(),
        }
    }

    fn convert_routing(&self, routing: &RoutingConfig) -> SingboxRoute {
        SingboxRoute {
            rules: routing
                .rules
                .iter()
                .map(|rule| SingboxRouteRule {
                    inbound: rule.inbound_tag.clone().unwrap_or_default(),
                    outbound: rule.outbound_tag.clone(),
                    domain: rule.domain.clone().unwrap_or_default(),
                    ip_cidr: rule.ip.clone().unwrap_or_default(),
                })
                .collect(),
        }
    }
}

#[async_trait::async_trait]
impl ProxyCore for SingboxCore {
    fn name(&self) -> &'static str {
        "sing-box"
    }

    async fn version(&self) -> Result<String, CoreError> {
        let output = Command::new("sing-box")
            .arg("version")
            .output()
            .map_err(|e| CoreError::ApiError(e.to_string()))?;

        let version = String::from_utf8_lossy(&output.stdout);
        Ok(version.trim().to_string())
    }

    async fn start(&mut self, config: CoreConfig) -> Result<(), CoreError> {
        if self.is_running() {
            return Err(CoreError::AlreadyRunning);
        }

        // Convert and write config
        let singbox_config = self.convert_config(&config);
        let config_json = serde_json::to_string_pretty(&singbox_config)
            .map_err(|e| CoreError::ConfigInvalid(e.to_string()))?;

        fs::write(&self.config_path, config_json)
            .await
            .map_err(|e| CoreError::StartFailed(e.to_string()))?;

        // Start sing-box
        let child = Command::new("sing-box")
            .arg("run")
            .arg("-c")
            .arg(&self.config_path)
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
        if let Some(mut process) = self.process.take() {
            process
                .kill()
                .map_err(|e| CoreError::StopFailed(e.to_string()))?;
            self.config = None;
            Ok(())
        } else {
            Err(CoreError::NotRunning)
        }
    }

    fn is_running(&self) -> bool {
        if let Some(process) = &self.process {
            process.id() > 0
        } else {
            false
        }
    }

    fn supported_protocols(&self) -> Vec<Protocol> {
        vec![
            Protocol::Vmess,
            Protocol::Vless,
            Protocol::Trojan,
            Protocol::Shadowsocks,
            Protocol::Hysteria2, // Sing-box exclusive
            Protocol::Tuic,      // Sing-box exclusive
            Protocol::Socks,
            Protocol::Http,
            Protocol::Freedom,
            Protocol::Blackhole,
        ]
    }

    async fn validate_config(&self, config: &CoreConfig) -> Result<(), CoreError> {
        // Check for unsupported protocols
        for inbound in &config.inbounds {
            if !self.supported_protocols().contains(&inbound.protocol) {
                return Err(CoreError::ProtocolNotSupported(
                    inbound.protocol.display_name().to_string(),
                ));
            }
        }

        Ok(())
    }

    async fn get_stats(&self) -> Result<CoreStats, CoreError> {
        if !self.is_running() {
            return Err(CoreError::NotRunning);
        }

        // In production, would query Sing-box API
        Ok(CoreStats {
            uptime_seconds: 0,
            total_upload_bytes: 0,
            total_download_bytes: 0,
            active_connections: 0,
            user_stats: HashMap::new(),
        })
    }

    async fn add_user(&mut self, user: UserConfig) -> Result<(), CoreError> {
        if !self.is_running() {
            return Err(CoreError::NotRunning);
        }

        // In production, would use Sing-box API to add user dynamically
        Ok(())
    }

    async fn remove_user(&mut self, user_id: &str) -> Result<(), CoreError> {
        if !self.is_running() {
            return Err(CoreError::NotRunning);
        }

        // In production, would use Sing-box API
        Ok(())
    }

    async fn update_user(&mut self, user_id: &str, user: UserConfig) -> Result<(), CoreError> {
        self.remove_user(user_id).await?;
        self.add_user(user).await
    }
}

/// Sing-box configuration format
#[derive(Debug, Serialize, Deserialize)]
struct SingboxConfig {
    log: SingboxLog,
    inbounds: Vec<SingboxInbound>,
    outbounds: Vec<SingboxOutbound>,
    route: SingboxRoute,
}

#[derive(Debug, Serialize, Deserialize)]
struct SingboxLog {
    level: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SingboxInbound {
    tag: String,
    r#type: String,
    listen: String,
    listen_port: u16,
    users: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SingboxOutbound {
    tag: String,
    r#type: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SingboxRoute {
    rules: Vec<SingboxRouteRule>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SingboxRouteRule {
    inbound: String,
    outbound: String,
    domain: Vec<String>,
    ip_cidr: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_singbox_core_creation() {
        let core = SingboxCore::new();
        assert_eq!(core.name(), "sing-box");
        assert!(!core.is_running());
    }

    #[test]
    fn test_supported_protocols() {
        let core = SingboxCore::new();
        let protocols = core.supported_protocols();

        assert!(protocols.contains(&Protocol::Hysteria2));
        assert!(protocols.contains(&Protocol::Tuic));
        assert!(protocols.contains(&Protocol::Vmess));
    }

    #[tokio::test]
    async fn test_validate_config() {
        let core = SingboxCore::new();

        let config = CoreConfig {
            log_level: LogLevel::Info,
            inbounds: vec![InboundConfig {
                tag: "test".to_string(),
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
