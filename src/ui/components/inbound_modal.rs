//! Inbound Modal Component
//!
//! Modal for adding/editing inbound configurations with protocol-specific forms.

use crate::ui::components::forms::*;
use crate::ui::components::modal::Modal;
use crate::ui::components::secret_generator::{SecretGenerator, SecretType, generate_uuid_v4};
use crate::ui::components::transport_matrix::{
    NetworkType, ProtocolType, SecurityType, TransportMatrix,
};
use dioxus::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};

#[derive(Props, Clone, PartialEq)]
pub struct InboundModalProps {
    /// Whether the modal is open
    pub open: Signal<bool>,
    /// Optional inbound ID for editing (None for new inbound)
    #[props(default)]
    pub inbound_id: Option<i64>,
    /// Called when modal is closed
    #[props(default)]
    pub on_close: Option<EventHandler<()>>,
    /// Called when inbound is saved
    #[props(default)]
    pub on_save: Option<EventHandler<()>>,
}

#[component]
pub fn InboundModal(props: InboundModalProps) -> Element {
    let mut open = props.open;
    let mut active_tab = use_signal(|| "general");

    // Form state - Basic
    let mut remark = use_signal(|| String::new());
    let mut protocol = use_signal(|| None::<String>);
    let mut listen_ip = use_signal(|| String::from("0.0.0.0"));
    let mut port = use_signal(|| 8443i64);

    // Transport state - using new TransportMatrix types
    let mut network = use_signal(|| NetworkType::Tcp);
    let mut security = use_signal(|| SecurityType::None);

    // Port validation state
    let mut port_error = use_signal(|| None::<String>);

    // Protocol-specific credentials
    let mut client_uuid = use_signal(|| generate_uuid_v4());
    let mut client_password = use_signal(|| String::new());
    // REALITY settings
    let mut reality_dest = use_signal(|| String::new());
    let mut reality_sni = use_signal(|| vec![String::from("example.com")]);
    let mut reality_short_ids = use_signal(|| vec![String::new()]);
    let mut reality_private_key = use_signal(|| String::new());
    let mut reality_public_key = use_signal(|| String::new());
    let mut reality_pqc = use_signal(|| None::<String>);
    let mut tls_fingerprint = use_signal(|| String::from("chrome"));
    let mut stealth_handshake = use_signal(|| false);

    // WebSocket settings
    let mut ws_path = use_signal(|| String::from("/"));
    let mut ws_host = use_signal(|| String::new());

    // QoS & Speed Settings (kbps)
    let mut up_speed_limit = use_signal(|| 0i64);
    let mut down_speed_limit = use_signal(|| 0i64);

    // gRPC settings
    let mut grpc_service_name = use_signal(|| String::new());
    let mut grpc_multi_mode = use_signal(|| false);

    // TCP HTTP settings
    let mut tcp_path = use_signal(|| String::new());
    let mut tcp_host = use_signal(|| String::new());
    let mut tcp_type = use_signal(|| None::<String>);

    // Missing signals
    let mut flow_type = use_signal(|| None::<String>);
    let mut ss_method = use_signal(|| None::<String>);

    // Flow-J Settings
    let mut flow_j_padding = use_signal(|| 0i64);
    let mut flow_j_jitter = use_signal(|| 0i64);
    // Flow-J Advanced
    let mut flow_j_port_count = use_signal(|| 64i64);
    let mut flow_j_port_type = use_signal(|| Some("random_dynamic".to_string()));
    let mut flow_j_congestion = use_signal(|| Some("bbr".to_string()));
    let mut flow_j_rotation = use_signal(|| 60i64);
    let mut flow_j_multiport_enabled = use_signal(|| true);

    // DbMimic Settings
    let mut db_mimic_target = use_signal(|| Some("postgresql".to_string()));
    let mut db_mimic_name = use_signal(|| String::new());
    let mut db_mimic_user = use_signal(|| String::new());
    let mut db_mimic_payload = use_signal(|| String::new());

    // Slipstream Settings
    let mut slip_domain = use_signal(|| String::new());
    let mut slip_record = use_signal(|| Some("txt".to_string()));
    let mut slip_frag = use_signal(|| 1280i64);

    let protocol_options = vec![
        ChoiceBoxOption {
            value: "vless".to_string(),
            label: "VLESS".to_string(),
            description: Some("Lightweight protocol with XTLS support".to_string()),
            icon: None,
        },
        ChoiceBoxOption {
            value: "vmess".to_string(),
            label: "VMess".to_string(),
            description: Some("Original V2Ray protocol".to_string()),
            icon: None,
        },
        ChoiceBoxOption {
            value: "trojan".to_string(),
            label: "Trojan".to_string(),
            description: Some("TLS-based protocol".to_string()),
            icon: None,
        },
        ChoiceBoxOption {
            value: "flowj".to_string(),
            label: "Flow-J".to_string(),
            description: Some("Multi-modal stealth protocol".to_string()),
            icon: None,
        },
        ChoiceBoxOption {
            value: "hysteria2".to_string(),
            label: "Hysteria 2".to_string(),
            description: Some("High performance QUIC protocol".to_string()),
            icon: None,
        },
        ChoiceBoxOption {
            value: "tun".to_string(),
            label: "TUN".to_string(),
            description: Some("Kernel interface mode".to_string()),
            icon: None,
        },
    ];

    let ss_method_options = vec![
        ChoiceBoxOption {
            value: "2022-blake3-aes-128-gcm".to_string(),
            label: "2022-blake3-aes-128-gcm".to_string(),
            description: None,
            icon: None,
        },
        ChoiceBoxOption {
            value: "2022-blake3-aes-256-gcm".to_string(),
            label: "2022-blake3-aes-256-gcm".to_string(),
            description: None,
            icon: None,
        },
        ChoiceBoxOption {
            value: "2022-blake3-chacha20-poly1305".to_string(),
            label: "2022-blake3-chacha20-poly1305".to_string(),
            description: None,
            icon: None,
        },
        ChoiceBoxOption {
            value: "aes-256-gcm".to_string(),
            label: "aes-256-gcm".to_string(),
            description: None,
            icon: None,
        },
        ChoiceBoxOption {
            value: "chacha20-poly1305".to_string(),
            label: "chacha20-poly1305".to_string(),
            description: None,
            icon: None,
        },
    ];

    let flow_options = vec![
        ChoiceBoxOption {
            value: "".to_string(),
            label: "None".to_string(),
            description: None,
            icon: None,
        },
        ChoiceBoxOption {
            value: "xtls-rprx-vision".to_string(),
            label: "xtls-rprx-vision".to_string(),
            description: Some("Recommended for VLESS".to_string()),
            icon: None,
        },
        ChoiceBoxOption {
            value: "xtls-rprx-vision-udp443".to_string(),
            label: "xtls-rprx-vision-udp443".to_string(),
            description: None,
            icon: None,
        },
    ];

    let tcp_type_options = vec![
        ChoiceBoxOption {
            value: "none".to_string(),
            label: "None".to_string(),
            description: None,
            icon: None,
        },
        ChoiceBoxOption {
            value: "http".to_string(),
            label: "HTTP".to_string(),
            description: Some("HTTP header obfuscation".to_string()),
            icon: None,
        },
    ];

    let pqc_options = vec![
        ChoiceBoxOption {
            value: "".to_string(),
            label: "None".to_string(),
            description: None,
            icon: None,
        },
        ChoiceBoxOption {
            value: "Kyber768".to_string(),
            label: "Kyber-768".to_string(),
            description: Some("NIST Level 3 KEM".to_string()),
            icon: None,
        },
        ChoiceBoxOption {
            value: "Kyber1024".to_string(),
            label: "Kyber-1024".to_string(),
            description: Some("NIST Level 5 KEM".to_string()),
            icon: None,
        },
        ChoiceBoxOption {
            value: "Dilithium2".to_string(),
            label: "Dilithium-2".to_string(),
            description: Some("NIST Level 2 Sig".to_string()),
            icon: None,
        },
        ChoiceBoxOption {
            value: "Dilithium3".to_string(),
            label: "Dilithium-3".to_string(),
            description: Some("NIST Level 3 Sig".to_string()),
            icon: None,
        },
        ChoiceBoxOption {
            value: "Dilithium5".to_string(),
            label: "Dilithium-5".to_string(),
            description: Some("NIST Level 5 Sig".to_string()),
            icon: None,
        },
    ];

    // Get protocol type for compatibility filtering
    let current_protocol = protocol().as_ref().and_then(|p| match p.as_str() {
        "vless" => Some(ProtocolType::Vless),
        "vmess" => Some(ProtocolType::Vmess),
        "trojan" => Some(ProtocolType::Trojan),
        "shadowsocks" => Some(ProtocolType::Shadowsocks),
        "hysteria2" => Some(ProtocolType::Hysteria2),
        "flowj" => Some(ProtocolType::FlowJ),
        "tun" => Some(ProtocolType::Tun),
        _ => None,
    });

    // Check if protocol uses UUID or password
    let uses_uuid = matches!(
        protocol().as_deref(),
        Some("vless") | Some("vmess") | Some("flowj")
    );
    let uses_password = matches!(
        protocol().as_deref(),
        Some("trojan") | Some("shadowsocks") | Some("hysteria2")
    );
    let is_vless = protocol().as_deref() == Some("vless");

    // Auto-reset transport/security when protocol changes to incompatible types
    use_effect(move || {
        if let Some(proto) = current_protocol {
            let compatible_nets = proto.compatible_networks();
            if !compatible_nets.contains(&network()) {
                network.set(compatible_nets[0]);
            }

            let compatible_sec = proto.compatible_security(network());
            if !compatible_sec.contains(&security()) {
                security.set(compatible_sec[0]);
            }
        }
    });

    // Reality key generation using SecretGenerator
    let generate_keys = {
        let mut private_key = reality_private_key.clone();
        let mut public_key = reality_public_key.clone();
        move |_| {
            use crate::ui::components::secret_generator::generate_reality_keypair;
            let (priv_key, pub_key) = generate_reality_keypair();
            private_key.set(priv_key);
            public_key.set(pub_key);
        }
    };

    let handle_save = move |_| {
        // Reset errors
        port_error.set(None);

        // Validation
        let current_port = *port.read();
        if current_port < 1 || current_port > 65535 {
            port_error.set(Some("Port must be between 1 and 65535".to_string()));
            return;
        }

        // Flow-J Validation
        if protocol().as_deref() == Some("flowj") {
            let count = *flow_j_port_count.read();
            let p_type = flow_j_port_type.read();

            if p_type.as_deref() == Some("static_range") {
                if current_port + count - 1 > 65535 {
                    port_error.set(Some(format!(
                        "Port range {} - {} exceeds 65535",
                        current_port,
                        current_port + count - 1
                    )));
                    return;
                }
            }
        }

        if let Some(ref handler) = props.on_save {
            handler.call(());
        }
        open.set(false);
    };

    rsx! {
        Modal {
            open: props.open,
            title: if props.inbound_id.is_some() { "Edit Inbound".to_string() } else { "Add Inbound".to_string() },
            width: "900px".to_string(),
            on_close: props.on_close.clone(),
            footer: rsx! {
                div { class: "flex items-center justify-between w-full",
                     div { class: "flex gap-2",
                         // Optional: Reset button
                     }
                     div { class: "flex items-center gap-3",
                        button {
                            class: "px-4 py-2 bg-bg-tertiary hover:bg-white/5 border border-border text-blue-400 hover:text-blue-300 text-sm font-medium rounded transition flex items-center gap-2",
                            onclick: move |_| {
                                // Placeholder for share logic
                                // generate_qr_code(format!("{{"protocol": "{}", ...}}", protocol().unwrap_or_default()));
                            },
                            span { class: "material-symbols-outlined text-[16px]", "qr_code" }
                            "Share"
                        }
                        button {
                            class: "px-4 py-2 bg-transparent hover:bg-white/5 border border-border text-gray-300 text-sm font-medium rounded transition",
                            onclick: move |_| open.set(false),
                            "Cancel"
                        }
                        button {
                            class: "px-4 py-2 bg-primary hover:bg-primary-hover text-white text-sm font-medium rounded transition shadow-sm",
                            onclick: handle_save,
                            "Save"
                        }
                    }
                }
            },

            div { class: "flex flex-col h-[70vh]",
                // Tabs Header
                div { class: "flex border-b border-border bg-white/5 px-2",
                     for tab in &["general", "credentials", "transport", "security", "advanced"] {
                         button {
                             class: format!(
                                 "px-6 py-3 text-sm font-medium border-b-2 transition-colors {}",
                                 if active_tab() == *tab { "border-primary text-white bg-white/5" } else { "border-transparent text-gray-400 hover:text-gray-200" }
                             ),
                             onclick: move |_| active_tab.set(tab),
                             "{tab.to_uppercase()}"
                         }
                     }
                }

                // Scrollable Content
                div { class: "flex-1 overflow-y-auto p-6 space-y-6",

                    // --- GENERAL TAB ---
                    if active_tab() == "general" {
                        div { class: "space-y-4 animate-fade-in",
                            h3 { class: "text-lg font-semibold text-white border-b border-border pb-2 flex items-center gap-2",
                                span { class: "material-symbols-outlined text-primary", "settings" }
                                "Basic Information"
                            }

                            TextInput {
                                label: Some("Remark".to_string()),
                                value: remark,
                                placeholder: Some("Connection name...".to_string()),
                                required: true,
                            }

                            div { class: "grid grid-cols-2 gap-4",
                                ChoiceBox {
                                    label: Some("Protocol".to_string()),
                                    value: protocol,
                                    options: protocol_options,
                                    required: true,
                                }

                                div {
                                    NumberInput {
                                        label: Some("Port".to_string()),
                                        value: port,
                                        min: Some(1024i64),
                                        max: Some(65535i64),
                                        required: true,
                                    }
                                    if let Some(ref err) = port_error() {
                                        div { class: "text-xs text-red-400 mt-1 flex items-center gap-1",
                                            span { class: "material-symbols-outlined text-[14px]", "error" }
                                            "{err}"
                                        }
                                    }
                                }
                            }

                            TextInput {
                                label: Some("Listen IP".to_string()),
                                value: listen_ip,
                                description: Some("Leave as 0.0.0.0 for all interfaces".to_string()),
                            }
                        }
                    } // End General

                    // --- CREDENTIALS TAB ---
                    if active_tab() == "credentials" {
                        // Protocol-Specific Credentials
                        if protocol().is_some() {
                            div { class: "space-y-4 animate-fade-in",
                                h3 { class: "text-lg font-semibold text-white border-b border-border pb-2 flex items-center gap-2",
                                    span { class: "material-symbols-outlined text-primary", "key" }
                                    "Client Credentials"
                                }

                                if is_vless {
                                    VlessForm {
                                        uuid: client_uuid,
                                        flow: flow_type,
                                    }
                                     FlowJForm {
                                        uuid: client_uuid,
                                        padding: flow_j_padding,
                                        jitter: flow_j_jitter,
                                        port_count: flow_j_port_count,
                                        port_type: flow_j_port_type,
                                        congestion: flow_j_congestion,
                                        rotation_frequency: flow_j_rotation,
                                        multiport_enabled: flow_j_multiport_enabled,
                                    }
                                } else if uses_uuid {
                                     SecretGenerator {
                                        label: "Client UUID".to_string(),
                                        secret_type: SecretType::Uuid,
                                        value: client_uuid,
                                    }
                                }

                                if uses_password {
                                    TextInput {
                                        label: Some("Password".to_string()),
                                        value: client_password,
                                        placeholder: Some("Enter password...".to_string()),
                                        required: true,
                                    }
                                }

                                if protocol().as_deref() == Some("shadowsocks") {
                                    ChoiceBox {
                                        label: Some("Encryption Method".to_string()),
                                        value: ss_method,
                                        options: ss_method_options,
                                        required: true,
                                    }
                                }
                            }
                        } else {
                             div { class: "text-center text-gray-500 py-10", "Select a protocol in General tab first." }
                        }
                    } // End Credentials

                    // --- TRANSPORT TAB ---
                    if active_tab() == "transport" {
                        // Transport Settings with TransportMatrix
                        if protocol().is_some() {
                            div { class: "space-y-4 animate-fade-in",
                                h3 { class: "text-lg font-semibold text-white border-b border-border pb-2 flex items-center gap-2",
                                    span { class: "material-symbols-outlined text-primary", "router" }
                                    "Transport Settings"
                                }

                                TransportMatrix {
                                    network: network,
                                    security: security,
                                    protocol: current_protocol,
                                }

                                // TCP HTTP Settings (conditional)
                                if network() == NetworkType::Tcp {
                                    div { class: "space-y-4 p-4 bg-[#1a1a1a] rounded border border-border",
                                        h4 { class: "text-sm font-semibold text-gray-300 mb-3 flex items-center gap-2",
                                            span { class: "material-symbols-outlined text-[16px]", "http" }
                                            "TCP Settings"
                                        }

                                        ChoiceBox {
                                            label: Some("Header Type".to_string()),
                                            value: tcp_type,
                                            options: tcp_type_options,
                                        }

                                        if tcp_type() == Some("http".to_string()) {
                                            div { class: "grid grid-cols-2 gap-4",
                                                TextInput {
                                                    label: Some("Path".to_string()),
                                                    value: tcp_path,
                                                    placeholder: Some("/".to_string()),
                                                }

                                                TextInput {
                                                    label: Some("Host".to_string()),
                                                    value: tcp_host,
                                                    placeholder: Some("www.example.com".to_string()),
                                                }
                                            }
                                        }
                                    }
                                }

                                // WebSocket Settings (conditional)
                                if network() == NetworkType::WebSocket {
                                    div { class: "space-y-4 p-4 bg-[#1a1a1a] rounded border border-border",
                                        h4 { class: "text-sm font-semibold text-gray-300 mb-3 flex items-center gap-2",
                                            span { class: "material-symbols-outlined text-[16px]", "dynamic_feed" }
                                            "WebSocket Settings"
                                        }

                                        div { class: "grid grid-cols-2 gap-4",
                                            TextInput {
                                                label: Some("Path".to_string()),
                                                value: ws_path,
                                                placeholder: Some("/".to_string()),
                                            }

                                            TextInput {
                                                label: Some("Host".to_string()),
                                                value: ws_host,
                                                placeholder: Some("example.com".to_string()),
                                            }
                                        }
                                    }
                                }

                                // gRPC Settings (conditional)
                                if network() == NetworkType::Grpc {
                                    div { class: "space-y-4 p-4 bg-[#1a1a1a] rounded border border-border",
                                        h4 { class: "text-sm font-semibold text-gray-300 mb-3 flex items-center gap-2",
                                            span { class: "material-symbols-outlined text-[16px]", "api" }
                                            "gRPC Settings"
                                        }

                                        TextInput {
                                            label: Some("Service Name".to_string()),
                                            value: grpc_service_name,
                                            placeholder: Some("GunService".to_string()),
                                        }

                                        Switch {
                                            label: Some("Multi Mode".to_string()),
                                            value: grpc_multi_mode,
                                        }
                                    }
                                }

                                // DbMimic Settings
                                if network() == NetworkType::DbMimic {
                                    DbMimicForm {
                                        target: db_mimic_target,
                                        fake_db_name: db_mimic_name,
                                        fake_user: db_mimic_user,
                                        startup_payload_hex: db_mimic_payload,
                                    }
                                }

                                // Slipstream Settings
                                if network() == NetworkType::Slipstream {
                                    SlipstreamForm {
                                        root_domain: slip_domain,
                                        record_type: slip_record,
                                        udp_frag_limit: slip_frag,
                                    }
                                }
                            }
                         } else {
                             div { class: "text-center text-gray-500 py-10", "Select a protocol in General tab first." }
                        }
                    } // End Transport

                    // --- SECURITY TAB ---
                    if active_tab() == "security" {
                        // REALITY Settings (conditional)
                        if security() == SecurityType::Reality {
                            RealityForm {
                                dest: reality_dest,
                                sni: reality_sni,
                                short_ids: reality_short_ids,
                                private_key: reality_private_key,
                                public_key: reality_public_key,
                                pqc: reality_pqc,
                                fingerprint: tls_fingerprint,
                                stealth: stealth_handshake,
                                generate_keys: generate_keys,
                            }
                        }

                        // TLS Settings (conditional)
                        if security() == SecurityType::Tls {
                            div { class: "space-y-4 animate-fade-in",
                                h3 { class: "text-lg font-semibold text-white border-b border-border pb-2 flex items-center gap-2",
                                    span { class: "material-symbols-outlined text-blue-400", "lock" }
                                    "TLS Settings"
                                }

                                div { class: "p-4 bg-[#1a1a1a] rounded border border-border space-y-4",
                                    p { class: "text-sm text-gray-400",
                                        "TLS configuration will use auto-generated certificates or ACME-provided certificates from the panel settings."
                                    }

                                    ChoiceBox {
                                        label: Some("PQC Cipher".to_string()),
                                        value: reality_pqc,
                                        options: pqc_options,
                                        description: Some("Post-Quantum Overlay for standard TLS".to_string()),
                                    }

                                    ChoiceBox {
                                        label: Some("uTLS Fingerprint".to_string()),
                                        value: tls_fingerprint,
                                        options: vec![
                                            ChoiceBoxOption { value: "chrome".to_string(), label: "Chrome".to_string(), description: None, icon: None },
                                            ChoiceBoxOption { value: "firefox".to_string(), label: "Firefox".to_string(), description: None, icon: None },
                                            ChoiceBoxOption { value: "safari".to_string(), label: "Safari".to_string(), description: None, icon: None },
                                            ChoiceBoxOption { value: "randomized".to_string(), label: "Randomized".to_string(), description: None, icon: None },
                                        ],
                                        description: Some("Client Hello camouflage".to_string()),
                                    }
                                }
                            }
                        }
                        if security() == SecurityType::None {
                            div { class: "text-center text-gray-500 py-10", "No security enabled. Select TLS or Reality in Transport tab." }
                        }
                    } // End Security

                    // --- ADVANCED TAB ---
                    if active_tab() == "advanced" {
                        // QoS & Speed Section
                        div { class: "space-y-4 animate-fade-in",
                            h3 { class: "text-lg font-semibold text-white border-b border-border pb-2 flex items-center gap-2",
                                span { class: "material-symbols-outlined text-purple-400", "speed" }
                                "QoS & Speed"
                            }

                            div { class: "grid grid-cols-2 gap-4",
                                NumberInput {
                                    label: Some("Upload Limit (kbps)".to_string()),
                                    value: up_speed_limit,
                                    min: Some(0i64),
                                    description: Some("0 for unlimited".to_string()),
                                }

                                NumberInput {
                                    label: Some("Download Limit (kbps)".to_string()),
                                    value: down_speed_limit,
                                    min: Some(0i64),
                                    description: Some("0 for unlimited".to_string()),
                                }
                            }
                        }
                    }
                } // End Scrollable Content
            } // End Flex Col
        }
    }
}
