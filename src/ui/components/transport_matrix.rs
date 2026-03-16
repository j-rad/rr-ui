//! Transport Matrix Component
//!
//! Visual toggle grid for selecting network transport and security options.
//! Provides intuitive button-based selection with protocol compatibility awareness.

use dioxus::prelude::*;

/// Transport network type
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NetworkType {
    Tcp,
    WebSocket,
    Grpc,
    Http,
    Quic,
    DbMimic,
    Slipstream,
}

impl NetworkType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Tcp => "tcp",
            Self::WebSocket => "ws",
            Self::Grpc => "grpc",
            Self::Http => "http",
            Self::Quic => "quic",
            Self::DbMimic => "db_mimic",
            Self::Slipstream => "slipstream",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Tcp => "TCP",
            Self::WebSocket => "WebSocket",
            Self::Grpc => "gRPC",
            Self::Http => "HTTP",
            Self::Quic => "QUIC",
            Self::DbMimic => "DB Mimic",
            Self::Slipstream => "Slipstream+",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Tcp => "lan",
            Self::WebSocket => "dynamic_feed",
            Self::Grpc => "api",
            Self::Http => "http",
            Self::Quic => "speed",
            Self::DbMimic => "database",
            Self::Slipstream => "dns",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::Tcp,
            Self::WebSocket,
            Self::Grpc,
            Self::Http,
            Self::Quic,
            Self::DbMimic,
            Self::Slipstream,
        ]
    }
}

/// Security type
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SecurityType {
    #[default]
    None,
    Tls,
    Reality,
}

impl SecurityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Tls => "tls",
            Self::Reality => "reality",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Tls => "TLS",
            Self::Reality => "REALITY",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::None => "lock_open",
            Self::Tls => "lock",
            Self::Reality => "verified_user",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![Self::None, Self::Tls, Self::Reality]
    }
}

/// Protocol type for compatibility filtering
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProtocolType {
    Vless,
    Vmess,
    Trojan,
    Shadowsocks,
    Hysteria2,
    FlowJ,
    Tun,
}

impl ProtocolType {
    /// Get compatible network types for this protocol
    pub fn compatible_networks(&self) -> Vec<NetworkType> {
        match self {
            Self::Vless | Self::Vmess => vec![
                NetworkType::Tcp,
                NetworkType::WebSocket,
                NetworkType::Grpc,
                NetworkType::Http,
                NetworkType::Quic,
                NetworkType::DbMimic,
                NetworkType::Slipstream,
            ],
            Self::Trojan => vec![NetworkType::Tcp, NetworkType::WebSocket, NetworkType::Grpc],
            Self::Shadowsocks => vec![NetworkType::Tcp],
            Self::Hysteria2 => vec![NetworkType::Quic],
            Self::FlowJ => vec![NetworkType::Tcp], // Flow-J handles its own transport abstraction
            Self::Tun => vec![NetworkType::Tcp],   // Tun is virtual
        }
    }

    /// Get compatible security types for this protocol
    pub fn compatible_security(&self, network: NetworkType) -> Vec<SecurityType> {
        match (self, network) {
            // VLESS/Vmess with TCP can use any security
            (Self::Vless | Self::Vmess, NetworkType::Tcp) => {
                vec![SecurityType::None, SecurityType::Tls, SecurityType::Reality]
            }
            // VLESS/Vmess with WS/gRPC/HTTP can use None or TLS
            (
                Self::Vless | Self::Vmess,
                NetworkType::WebSocket | NetworkType::Grpc | NetworkType::Http,
            ) => {
                vec![SecurityType::None, SecurityType::Tls]
            }
            // Trojan requires TLS
            (Self::Trojan, _) => vec![SecurityType::Tls],
            // Shadowsocks doesn't use external TLS
            (Self::Shadowsocks, _) => vec![SecurityType::None],
            // Hysteria2 uses built-in QUIC TLS
            (Self::Hysteria2, _) => vec![SecurityType::Tls],
            // Flow-J supports specialized obfuscation and Reality
            (Self::FlowJ, _) => vec![SecurityType::None, SecurityType::Reality],
            // Tun doesn't use transport security
            (Self::Tun, _) => vec![SecurityType::None],
            // Default fallback
            _ => vec![SecurityType::None, SecurityType::Tls],
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct TransportMatrixProps {
    /// Selected network type
    pub network: Signal<NetworkType>,
    /// Selected security type
    pub security: Signal<SecurityType>,
    /// Optional protocol for compatibility filtering
    #[props(default)]
    pub protocol: Option<ProtocolType>,
    /// Disable interaction
    #[props(default = false)]
    pub disabled: bool,
}

/// Visual transport matrix for network/security selection
#[component]
pub fn TransportMatrix(props: TransportMatrixProps) -> Element {
    let mut network = props.network;
    let mut security = props.security;

    let available_networks = props
        .protocol
        .map(|p| p.compatible_networks())
        .unwrap_or_else(NetworkType::all);

    let available_security = props
        .protocol
        .map(|p| p.compatible_security(network()))
        .unwrap_or_else(SecurityType::all);

    rsx! {
        div { class: "space-y-4",
            // Network Selection
            div { class: "space-y-2",
                label { class: "block text-sm font-medium text-gray-300", "Network Transport" }
                div { class: "flex flex-wrap gap-2",
                    for net in available_networks.iter() {
                        {
                            let net = *net;
                            let is_selected = network() == net;
                            let disabled = props.disabled;
                            rsx! {
                                button {
                                    key: "{net.as_str()}",
                                    r#type: "button",
                                    disabled: disabled,
                                    class: "flex items-center gap-2 px-4 py-2.5 rounded-lg border transition-all text-sm font-medium",
                                    class: if is_selected {
                                        "bg-primary border-primary text-white shadow-sm"
                                    } else {
                                        "bg-bg-tertiary border-border text-gray-400 hover:border-primary/50 hover:text-white"
                                    },
                                    onclick: move |_| {
                                        network.set(net);
                                    },
                                    span { class: "material-symbols-outlined text-[18px]", "{net.icon()}" }
                                    span { "{net.label()}" }
                                }
                            }
                        }
                    }
                }
            }

            // Security Selection
            div { class: "space-y-2",
                label { class: "block text-sm font-medium text-gray-300", "Security Layer" }
                div { class: "flex flex-wrap gap-2",
                    for sec in available_security.iter() {
                        {
                            let sec = *sec;
                            let is_selected = security() == sec;
                            let disabled = props.disabled;
                            let is_available = available_security.contains(&sec);
                            rsx! {
                                button {
                                    key: "{sec.as_str()}",
                                    r#type: "button",
                                    disabled: disabled || !is_available,
                                    class: "flex items-center gap-2 px-4 py-2.5 rounded-lg border transition-all text-sm font-medium",
                                    class: if is_selected {
                                        "bg-primary border-primary text-white shadow-sm"
                                    } else if !is_available {
                                        "bg-bg-tertiary border-border text-gray-600 cursor-not-allowed opacity-50"
                                    } else {
                                        "bg-bg-tertiary border-border text-gray-400 hover:border-primary/50 hover:text-white"
                                    },
                                    onclick: move |_| {
                                        if is_available {
                                            security.set(sec);
                                        }
                                    },
                                    span { class: "material-symbols-outlined text-[18px]", "{sec.icon()}" }
                                    span { "{sec.label()}" }
                                    if sec == SecurityType::Reality {
                                        span { class: "px-1.5 py-0.5 bg-green-500/20 text-green-400 text-[10px] rounded font-bold", "NEW" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_type_labels() {
        assert_eq!(NetworkType::Tcp.label(), "TCP");
        assert_eq!(NetworkType::WebSocket.as_str(), "ws");
    }

    #[test]
    fn test_security_type_labels() {
        assert_eq!(SecurityType::Reality.label(), "REALITY");
        assert_eq!(SecurityType::Tls.as_str(), "tls");
    }

    #[test]
    fn test_protocol_compatibility() {
        let vless_nets = ProtocolType::Vless.compatible_networks();
        assert!(vless_nets.contains(&NetworkType::Tcp));
        assert!(vless_nets.contains(&NetworkType::WebSocket));

        let ss_nets = ProtocolType::Shadowsocks.compatible_networks();
        assert!(ss_nets.contains(&NetworkType::Tcp));
        assert!(!ss_nets.contains(&NetworkType::WebSocket));
    }
}
