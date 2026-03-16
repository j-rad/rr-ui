use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::borrow::Cow;
use std::io::Write;

use crate::db::DbClient;
use crate::models::{Inbound, OutboundModel, StreamSettings};
use anyhow::Result;

/// Represents the main RustRay configuration.
#[derive(Debug, Serialize, Deserialize)]
pub struct RustRayConfig<'a> {
    pub log: LogConfig<'a>,
    pub api: ApiConfig<'a>,
    pub stats: Value,
    pub policy: PolicyConfig,
    pub inbounds: Vec<InboundConfig<'a>>,
    pub outbounds: Vec<OutboundConfig<'a>>,
    pub routing: RoutingConfig<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns: Option<Value>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub balancers: Vec<BalancerConfig<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observatory: Option<ObservatoryConfig<'a>>,
}

impl<'a> RustRayConfig<'a> {
    /// Writes the configuration to the given writer using serde_json::to_writer.
    /// This avoids creating a monolithic String in memory.
    pub fn write_to<W: Write>(&self, writer: W) -> serde_json::Result<()> {
        serde_json::to_writer(writer, self)
    }
}
// ... (LogConfig, ApiConfig, PolicyConfig, InboundConfig, OutboundConfig structs remain unchanged) -> REPLACING THIS

/// Represents the logging configuration.
#[derive(Debug, Serialize, Deserialize)]
pub struct LogConfig<'a> {
    pub access: Cow<'a, str>,
    pub error: Cow<'a, str>,
    pub loglevel: Cow<'a, str>,
}

/// Represents the API configuration.
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiConfig<'a> {
    pub tag: Cow<'a, str>,
    pub services: Vec<Cow<'a, str>>,
}

/// Represents the policy configuration.
#[derive(Debug, Serialize, Deserialize)]
pub struct PolicyConfig {
    pub levels: Value,
    pub system: Value,
}

/// Represents an inbound connection configuration.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboundConfig<'a> {
    pub tag: Cow<'a, str>,
    pub port: u32,
    pub protocol: Cow<'a, str>,
    pub settings: Value,
    pub stream_settings: Value,
    pub sniffing: Value,
}

/// Represents an outbound connection configuration.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutboundConfig<'a> {
    pub tag: Cow<'a, str>,
    pub protocol: Cow<'a, str>,
    pub settings: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_settings: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mux: Option<Value>,
}

/// Represents the routing configuration.
#[derive(Debug, Serialize, Deserialize)]
pub struct RoutingConfig<'a> {
    #[serde(rename = "domainStrategy")]
    pub domain_strategy: Cow<'a, str>,
    pub rules: Vec<Value>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub balancers: Vec<BalancerConfig<'a>>,
}

// ... (BalancerConfig, ObservatoryConfig, RoutingTemplate remain unchanged) -> REPLACING THIS

/// Represents a balancer configuration.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BalancerConfig<'a> {
    pub tag: Cow<'a, str>,
    pub selector: Vec<Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_tag: Option<Cow<'a, str>>,
}

/// Represents the observatory configuration for connection health checks.
#[derive(Debug, Serialize, Deserialize)]
pub struct ObservatoryConfig<'a> {
    #[serde(rename = "subjectSelector")]
    pub subject_selector: Vec<Cow<'a, str>>,
    #[serde(rename = "probeUrl")]
    pub probe_url: Cow<'a, str>,
    #[serde(rename = "probeInterval")]
    pub probe_interval: Cow<'a, str>,
}

pub struct RustRayConfigBuilder;

impl RustRayConfigBuilder {
    pub async fn build(db: &DbClient) -> Result<RustRayConfig<'static>> {
        #[cfg(feature = "server")]
        let inbound_models: Vec<Inbound<'static>> =
            db.client.select("inbound").await.unwrap_or_default();
        #[cfg(not(feature = "server"))]
        let inbound_models: Vec<Inbound<'static>> = vec![];

        #[cfg(feature = "server")]
        let outbound_models: Vec<OutboundModel<'static>> =
            db.client.select("outbound").await.unwrap_or_default();
        #[cfg(not(feature = "server"))]
        let outbound_models: Vec<OutboundModel<'static>> = vec![];

        // Fetch DNS config from DB
        #[cfg(feature = "server")]
        let dns_config: Option<crate::models::DnsConfig> =
            db.client.select(("sys_config", "dns")).await.ok().flatten();

        #[cfg(not(feature = "server"))]
        let dns_config: Option<crate::models::DnsConfig> = None;

        let dns_value = if let Some(dns) = dns_config {
            if dns.enabled {
                match serde_json::to_value(dns) {
                    Ok(mut value) => {
                        // Remove internal 'enabled' field if it exists in the JSON object
                        if let Value::Object(ref mut map) = value {
                            map.remove("enabled");
                        }
                        Some(value)
                    }
                    Err(_) => None,
                }
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self::build_owned(
            inbound_models,
            outbound_models,
            dns_value,
        ))
    }

    pub fn build_owned(
        inbound_models: Vec<Inbound<'static>>,
        outbound_models: Vec<OutboundModel<'static>>,
        dns: Option<Value>,
    ) -> RustRayConfig<'static> {
        let mut inbounds = vec![];
        for model in inbound_models {
            if model.enable {
                inbounds.push(Self::build_inbound_config_owned(model));
            }
        }

        inbounds.push(Self::build_api_inbound());

        let mut outbounds = Self::build_default_outbounds();

        for model in outbound_models {
            if model.enable {
                outbounds.push(Self::build_outbound_config_owned(model));
            }
        }

        let routing = Self::build_default_routing();

        RustRayConfig {
            log: LogConfig {
                access: Cow::Borrowed("access.log"),
                error: Cow::Borrowed("error.log"),
                loglevel: Cow::Borrowed("warning"),
            },
            api: ApiConfig {
                tag: Cow::Borrowed("api"),
                services: vec![
                    Cow::Borrowed("HandlerService"),
                    Cow::Borrowed("LoggerService"),
                    Cow::Borrowed("StatsService"),
                ],
            },
            stats: json!({}),
            policy: PolicyConfig {
                levels: json!({
                    "0": {
                        "handshake": 4,
                        "connIdle": 300,
                        "uplinkOnly": 2,
                        "downlinkOnly": 5,
                        "statsUserUplink": true,
                        "statsUserDownlink": true
                    }
                }),
                system: json!({
                    "statsInboundUplink": true,
                    "statsInboundDownlink": true,
                    "statsOutboundUplink": true,
                    "statsOutboundDownlink": true
                }),
            },
            inbounds,
            outbounds,
            routing,
            dns,
            balancers: vec![],
            observatory: None,
        }
    }

    pub fn build_from_models<'a>(
        inbound_models: &'a [Inbound<'a>],
        outbound_models: &'a [OutboundModel<'a>],
    ) -> RustRayConfig<'a> {
        let mut inbounds = vec![];
        for model in inbound_models {
            if model.enable {
                inbounds.push(Self::build_inbound_config(model));
            }
        }

        inbounds.push(Self::build_api_inbound());

        let mut outbounds = Self::build_default_outbounds();

        for model in outbound_models {
            if model.enable {
                outbounds.push(Self::build_outbound_config(model));
            }
        }

        let routing = Self::build_default_routing();

        RustRayConfig {
            log: LogConfig {
                access: Cow::Borrowed("access.log"),
                error: Cow::Borrowed("error.log"),
                loglevel: Cow::Borrowed("warning"),
            },
            api: ApiConfig {
                tag: Cow::Borrowed("api"),
                services: vec![
                    Cow::Borrowed("HandlerService"),
                    Cow::Borrowed("LoggerService"),
                    Cow::Borrowed("StatsService"),
                ],
            },
            stats: json!({}),
            policy: PolicyConfig {
                levels: json!({
                    "0": {
                        "handshake": 4,
                        "connIdle": 300,
                        "uplinkOnly": 2,
                        "downlinkOnly": 5,
                        "statsUserUplink": true,
                        "statsUserDownlink": true
                    }
                }),
                system: json!({
                    "statsInboundUplink": true,
                    "statsInboundDownlink": true,
                    "statsOutboundUplink": true,
                    "statsOutboundDownlink": true
                }),
            },
            inbounds,
            outbounds,
            routing,
            dns: None,
            balancers: vec![],
            observatory: None,
        }
    }

    fn build_inbound_config<'a>(model: &'a Inbound<'a>) -> InboundConfig<'a> {
        let (settings, stream_settings, sniffing, port, protocol_str) =
            Self::extract_inbound_fields(model);

        let protocol = if protocol_str == "tun" {
            Cow::Borrowed("dokodemo-door")
        } else {
            Cow::Owned(protocol_str)
        };

        InboundConfig {
            tag: Cow::Borrowed(model.tag.as_ref()),
            port,
            protocol,
            settings,
            stream_settings,
            sniffing,
        }
    }

    fn build_inbound_config_owned(model: Inbound<'static>) -> InboundConfig<'static> {
        let (settings, stream_settings, sniffing, port, protocol_str) =
            Self::extract_inbound_fields(&model);

        let protocol = if protocol_str == "tun" {
            Cow::Borrowed("dokodemo-door")
        } else {
            // If we are strictly owned, we might want to take ownership.
            // But extract_inbound_fields borrows 'model'.
            // To avoid clone, we should consume model.
            // But for now, cloning Cow (cheap if borrowed, expensive if owned) or just re-using owned.
            // In 'build_owned', 'model' is consumed. But here we have referenced it.
            // To optimize, 'extract_inbound_fields' should probably consume model or return parts.
            // But Cow::Owned(model.protocol.into_owned()) is what we do if we want full decouple.
            // However, model.protocol is Cow<'static>.
            Cow::Owned(model.protocol.to_string())
        };

        InboundConfig {
            tag: Cow::Owned(model.tag.into_owned()),
            port,
            protocol,
            settings,
            stream_settings,
            sniffing,
        }
    }

    fn extract_inbound_fields<'a>(model: &'a Inbound<'_>) -> (Value, Value, Value, u32, String) {
        use crate::models::ProtocolSettings;

        // Pattern match on the enum and extract the inner settings
        let mut settings = match &model.settings {
            ProtocolSettings::Vless(vless_settings) => {
                serde_json::to_value(vless_settings).unwrap_or_default()
            }
            ProtocolSettings::Vmess(vmess_settings) => {
                serde_json::to_value(vmess_settings).unwrap_or_default()
            }
            ProtocolSettings::Trojan(trojan_settings) => {
                serde_json::to_value(trojan_settings).unwrap_or_default()
            }
            ProtocolSettings::Shadowsocks(ss_settings) => {
                serde_json::to_value(ss_settings).unwrap_or_default()
            }
            ProtocolSettings::Hysteria2(hy2_settings) => {
                serde_json::to_value(hy2_settings).unwrap_or_default()
            }
            ProtocolSettings::Tuic(tuic_settings) => {
                serde_json::to_value(tuic_settings).unwrap_or_default()
            }
            ProtocolSettings::FlowJ(flowj_settings) => {
                serde_json::to_value(flowj_settings).unwrap_or_default()
            }
            ProtocolSettings::Naive(naive_settings) => {
                serde_json::to_value(naive_settings).unwrap_or_default()
            }
            ProtocolSettings::WireGuard(wg_settings) => {
                serde_json::to_value(wg_settings).unwrap_or_default()
            }
            ProtocolSettings::Tun(_tun_settings) => {
                json!({
                    "network": "tcp,udp",
                    "followRedirect": true,
                    "address": "127.0.0.1",
                })
            }
            ProtocolSettings::Socks(socks) => serde_json::to_value(socks).unwrap_or_default(),
            ProtocolSettings::Http(http) => serde_json::to_value(http).unwrap_or_default(),
            ProtocolSettings::Dokodemo(dokodemo) => {
                serde_json::to_value(dokodemo).unwrap_or_default()
            }
        };

        if model.protocol == crate::domain::models::InboundProtocol::Vless {
            if model.stream_settings.security == "reality" {
                settings["decryption"] = json!("none");
            } else if let Some(clients) = settings.get("clients").and_then(|c| c.as_array()) {
                if clients.iter().any(|c| {
                    let flow = c.get("flow").and_then(|f| f.as_str()).unwrap_or("");
                    flow == "xtls-rprx-vision" || flow == "xtls-rprx-vision-udp443"
                }) {
                    settings["decryption"] = json!("none");
                }
            }
        }

        let stream_settings = Self::build_stream_settings(&model.stream_settings);
        let sniffing = serde_json::to_value(&model.sniffing).unwrap_or_default();

        let port = if model.protocol == crate::domain::models::InboundProtocol::Dokodemo {
            12345
        } else {
            model.port
        };

        (
            settings,
            stream_settings,
            sniffing,
            port,
            model.protocol.to_string(),
        )
    }

    fn build_outbound_config<'a>(model: &'a OutboundModel<'a>) -> OutboundConfig<'a> {
        let (settings, stream_settings) = Self::extract_outbound_fields(model);

        OutboundConfig {
            tag: Cow::Borrowed(model.tag.as_ref()),
            protocol: Cow::Borrowed(model.protocol.as_ref()),
            settings,
            stream_settings,
            mux: model.mux.clone(),
        }
    }

    fn build_outbound_config_owned(model: OutboundModel<'static>) -> OutboundConfig<'static> {
        let (settings, stream_settings) = Self::extract_outbound_fields(&model);

        OutboundConfig {
            tag: Cow::Owned(model.tag.into_owned()),
            protocol: Cow::Owned(model.protocol.into_owned()),
            settings,
            stream_settings,
            mux: model.mux,
        }
    }

    fn extract_outbound_fields(model: &OutboundModel<'_>) -> (Value, Option<Value>) {
        use crate::models::OutboundSettings;

        let settings = match &model.settings {
            OutboundSettings::Freedom(s) => serde_json::to_value(s).unwrap_or_default(),
            OutboundSettings::Blackhole(s) => serde_json::to_value(s).unwrap_or_default(),
            OutboundSettings::Vless(s) => serde_json::to_value(s).unwrap_or_default(),
            OutboundSettings::Vmess(s) => serde_json::to_value(s).unwrap_or_default(),
            OutboundSettings::Trojan(s) => serde_json::to_value(s).unwrap_or_default(),
            OutboundSettings::Shadowsocks(s) => serde_json::to_value(s).unwrap_or_default(),
            OutboundSettings::Tailscale(s) => serde_json::to_value(s).unwrap_or_default(),
            OutboundSettings::Tor(s) => serde_json::to_value(s).unwrap_or_default(),
        };

        let stream_settings =
            if model.stream_settings.network == "" && model.stream_settings.security == "" {
                None
            } else {
                Some(Self::build_stream_settings(&model.stream_settings))
            };

        (settings, stream_settings)
    }

    fn build_stream_settings(settings: &StreamSettings<'_>) -> Value {
        let mut stream_settings = json!({
            "network": settings.network,
            "security": settings.security,
        });

        if settings.security == "reality" {
            if let Some(reality_settings) = &settings.reality_settings {
                stream_settings["realitySettings"] =
                    serde_json::to_value(reality_settings).unwrap_or_default();
            }
        } else if settings.security == "tls" {
            if let Some(tls_settings) = &settings.tls_settings {
                stream_settings["tlsSettings"] =
                    serde_json::to_value(tls_settings).unwrap_or_default();
            }
        }

        match settings.network.as_ref() {
            "ws" => {
                if let Some(ws_settings) = &settings.ws_settings {
                    stream_settings["wsSettings"] =
                        serde_json::to_value(ws_settings).unwrap_or_default();
                }
            }
            "http" => {
                if let Some(http_settings) = &settings.http_settings {
                    stream_settings["httpSettings"] =
                        serde_json::to_value(http_settings).unwrap_or_default();
                }
            }
            "kcp" => {
                if let Some(kcp_settings) = &settings.kcp_settings {
                    stream_settings["kcpSettings"] =
                        serde_json::to_value(kcp_settings).unwrap_or_default();
                }
            }
            "grpc" => {
                if let Some(grpc_settings) = &settings.grpc_settings {
                    stream_settings["grpcSettings"] =
                        serde_json::to_value(grpc_settings).unwrap_or_default();
                }
            }
            "tcp" => {
                if let Some(tcp_settings) = &settings.tcp_settings {
                    stream_settings["tcpSettings"] =
                        serde_json::to_value(tcp_settings).unwrap_or_default();
                }
            }
            _ => {}
        }

        stream_settings
    }

    fn build_api_inbound() -> InboundConfig<'static> {
        InboundConfig {
            tag: Cow::Borrowed("api"),
            port: 10085,
            protocol: Cow::Borrowed("dokodemo-door"),
            settings: json!({ "address": "127.0.0.1" }),
            stream_settings: json!({}),
            sniffing: json!({
                "enabled": false,
                "destOverride": ["http", "tls"]
            }),
        }
    }

    fn build_default_outbounds() -> Vec<OutboundConfig<'static>> {
        vec![
            OutboundConfig {
                tag: Cow::Borrowed("direct"),
                protocol: Cow::Borrowed("freedom"),
                settings: json!({}),
                stream_settings: None,
                mux: None,
            },
            OutboundConfig {
                tag: Cow::Borrowed("blocked"),
                protocol: Cow::Borrowed("blackhole"),
                settings: json!({}),
                stream_settings: None,
                mux: None,
            },
        ]
    }

    fn build_default_routing() -> RoutingConfig<'static> {
        RoutingConfig {
            domain_strategy: Cow::Borrowed("IPIfNonMatch"),
            rules: vec![json!({ "type": "field", "inbound_tag": ["api"], "outbound_tag": "api" })],
            balancers: vec![],
        }
    }
}

/// Predefined routing templates for different regions.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum RoutingTemplate {
    Default,
    ChinaOptimized,
    IranOptimized,
    RussiaOptimized,
}

impl RoutingTemplate {
    pub fn generate_rules(&self) -> Vec<Value> {
        match self {
            RoutingTemplate::Default => vec![
                json!({
                    "type": "field",
                    "outbound_tag": "blocked",
                    "ip": ["geoip:private"]
                }),
                json!({
                    "type": "field",
                    "outbound_tag": "direct",
                    "network": "udp,tcp"
                }),
            ],
            RoutingTemplate::ChinaOptimized => vec![
                json!({
                    "type": "field",
                    "outbound_tag": "blocked",
                    "domain": ["geosite:category-ads-all"]
                }),
                json!({
                    "type": "field",
                    "outbound_tag": "direct",
                    "domain": ["geosite:cn"],
                    "ip": ["geoip:cn", "geoip:private"]
                }),
                json!({
                    "type": "field",
                    "outbound_tag": "proxy",
                    "network": "tcp,udp"
                }),
            ],
            RoutingTemplate::IranOptimized => vec![
                json!({
                    "type": "field",
                    "outbound_tag": "blocked",
                    "domain": ["geosite:category-ads-all"]
                }),
                json!({
                    "type": "field",
                    "outbound_tag": "direct",
                    "domain": ["geosite:ir"],
                    "ip": ["geoip:ir", "geoip:private"]
                }),
                json!({
                    "type": "field",
                    "outbound_tag": "proxy",
                    "network": "tcp,udp"
                }),
            ],
            RoutingTemplate::RussiaOptimized => vec![
                json!({
                    "type": "field",
                    "outbound_tag": "blocked",
                    "domain": ["geosite:category-ads-all"]
                }),
                json!({
                    "type": "field",
                    "outbound_tag": "direct",
                    "domain": ["geosite:ru"],
                    "ip": ["geoip:ru", "geoip:private"]
                }),
                json!({
                    "type": "field",
                    "outbound_tag": "proxy",
                    "network": "tcp,udp"
                }),
            ],
        }
    }
}
