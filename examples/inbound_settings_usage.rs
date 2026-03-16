// Example demonstrating the new InboundSettings enum usage
// This file is for documentation purposes and shows how to use the refactored types

use rr_ui::models::{
    Client, FecSettings, FlowJMode, FlowJSettings, FlowType, Hysteria2Settings, Obfuscation,
    ProtocolSettings, RealitySettings, Shadowsocks2022Settings, VlessSettings, WireGuardSettings,
    WireguardPeer,
};

// Example 1: Creating a VLESS inbound
fn example_vless() {
    let vless_settings = VlessSettings {
        clients: vec![Client {
            id: Some("uuid-here".to_string()), // .into()
            email: Some("user@example.com".to_string()), // .into()
            flow: Some(FlowType::XtlsRprxVision),
            enable: true,
            total_flow_limit: 100,
            expiry_time: 0,
            ..Default::default()
        }],
        decryption: Some("none".to_string().into()),
        fallbacks: None,
        extra: Default::default(),
    };

    let inbound_settings = ProtocolSettings::Vless(vless_settings);

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&inbound_settings).unwrap();
    println!("VLESS JSON:\n{}\n", json);

    // Access clients using helper method
    if let Some(clients) = inbound_settings.clients() {
        println!("Number of clients: {}", clients.len());
    }
}

// Example 2: Creating a Shadowsocks inbound
fn example_shadowsocks() {
    let ss_settings = Shadowsocks2022Settings {
        method: "2022-blake3-aes-256-gcm".to_string().into(), // fixed
        password: Some("password123".to_string()),     // fixed
        key: None,
        network: Some("tcp,udp".to_string().into()), // fixed
        email: Some("admin@example.com".to_string().into()), // fixed
        level: Some(0),
        extra: Default::default(),
    };

    let inbound_settings = ProtocolSettings::Shadowsocks(ss_settings);

    let json = serde_json::to_string_pretty(&inbound_settings).unwrap();
    println!("Shadowsocks JSON:\n{}\n", json);

    // Shadowsocks doesn't have clients
    assert!(inbound_settings.clients().is_none());
}

// Example 3: Creating a Hysteria2 inbound
fn example_hysteria2() {
    let hy2_settings = Hysteria2Settings {
        up_mbps: Some(100),
        down_mbps: Some(200),
        password: Some("secure-password".to_string()),
        obfuscation: Some(Obfuscation {
            obfs_type: "salamander".to_string().into(),
            password: Some("obfs-password".to_string()),
        }),
        extra: Default::default(),
    };

    let inbound_settings = ProtocolSettings::Hysteria2(hy2_settings);

    let json = serde_json::to_string_pretty(&inbound_settings).unwrap();
    println!("Hysteria2 JSON:\n{}\n", json);
}

// Example 4: Creating a Flow-J inbound
fn example_flowj() {
    let flowj_settings = FlowJSettings {
        clients: vec![Client {
            id: Some("user-uuid".to_string()),
            email: Some("flowj-user@example.com".to_string()),
            level: Some(0),
            ..Default::default()
        }],
        mode: FlowJMode::Reality,
        mqtt: None,
        reality: Some(RealitySettings {
            show: true,
            dest: "example.com:443".to_string().into(),
            xver: 0,
            server_names: vec!["example.com".to_string().into()],
            private_key: "private-key-here".to_string().into(),
            short_ids: vec!["abcd".to_string().into()],
            fingerprint: "chrome".to_string().into(),
            stealth_handshake: true,
            ..Default::default()
        }),
        cdn: None,
        fec: Some(FecSettings {
            enabled: true,
            data_shards: 10,
            parity_shards: 3,
        }),
        stealth_handshake: true,
        flowj_config: None,
        extra: Default::default(),
    };

    let inbound_settings = ProtocolSettings::FlowJ(flowj_settings);

    let json = serde_json::to_string_pretty(&inbound_settings).unwrap();
    println!("Flow-J JSON:\n{}\n", json);
}

// Example 5: Creating a WireGuard inbound
fn example_wireguard() {
    let wg_settings = WireGuardSettings {
        secret_key: "secret-key-base64".to_string(),
        peers: vec![WireguardPeer {
            public_key: "peer-public-key".to_string(),
            allowed_ips: vec!["10.0.0.2/32".to_string()],
            endpoint: Some("peer.example.com:51820".to_string()),
            keep_alive: 25,
        }],
        mtu: Some(1420),
        reserved: Some([0, 0, 0]),
    };

    let inbound_settings = ProtocolSettings::WireGuard(wg_settings);

    let json = serde_json::to_string_pretty(&inbound_settings).unwrap();
    println!("WireGuard JSON:\n{}\n", json);
}

// Example 6: Pattern matching on ProtocolSettings
#[allow(dead_code)]
fn handle_inbound(settings: &ProtocolSettings) {
    match settings {
        ProtocolSettings::Vless(vless) => {
            println!("Processing VLESS with {} clients", vless.clients.len());
        }
        ProtocolSettings::Vmess(vmess) => {
            println!("Processing VMess with {} clients", vmess.clients.len());
        }
        ProtocolSettings::Trojan(trojan) => {
            println!("Processing Trojan with {} clients", trojan.clients.len());
        }
        ProtocolSettings::Shadowsocks(ss) => {
            println!("Processing Shadowsocks with method: {}", ss.method);
        }
        ProtocolSettings::Hysteria2(hy2) => {
            println!(
                "Processing Hysteria2 with bandwidth: up={:?}, down={:?}",
                hy2.up_mbps, hy2.down_mbps
            );
        }
        ProtocolSettings::Tuic(tuic) => {
            println!("Processing TUIC with {} users", tuic.users.len());
        }
        ProtocolSettings::FlowJ(flowj) => {
            println!("Processing Flow-J in {:?} mode", flowj.mode);
        }
        ProtocolSettings::Naive(_) => {
            println!("Processing Naive proxy");
        }
        ProtocolSettings::WireGuard(wg) => {
            println!("Processing WireGuard with {} peers", wg.peers.len());
        }
        ProtocolSettings::Tun(tun) => {
            println!("Processing TUN interface: {}", tun.interface);
        }
        ProtocolSettings::Socks(socks) => {
            println!("Processing Socks users: {}", socks.accounts.len());
        }
        ProtocolSettings::Http(http) => {
            println!("Processing HTTP users: {}", http.accounts.len());
        }
        ProtocolSettings::Dokodemo(doko) => {
            println!("Processing Dokodemo to: {}", doko.address);
        }
    }
}

// Example 7: Deserializing from JSON
fn example_deserialization() {
    let json = r#"{
        "protocol": "vless",
        "settings": {
            "clients": [
                {
                    "id": "uuid-123",
                    "email": "user@example.com",
                    "enable": true,
                    "totalGb": 100,
                    "expiryTime": 0
                }
            ],
            "decryption": "none"
        }
    }"#;

    let settings: ProtocolSettings = serde_json::from_str(json).unwrap();

    match &settings {
        ProtocolSettings::Vless(vless) => {
            println!("Deserialized VLESS with decryption: {:?}", vless.decryption);
            if let Some(clients) = settings.clients() {
                println!("Number of clients: {}", clients.len());
            }
        }
        _ => println!("Unexpected protocol"),
    }
}

// Example 8: Using helper methods
fn example_helper_methods() {
    let vless = ProtocolSettings::Vless(VlessSettings::default());
    let ss = ProtocolSettings::Shadowsocks(Shadowsocks2022Settings::default());

    // Get protocol name
    println!("Protocol 1: {}", vless.protocol_name());
    println!("Protocol 2: {}", ss.protocol_name());

    // Check if protocol supports clients
    println!("VLESS has clients: {}", vless.clients().is_some());
    println!("Shadowsocks has clients: {}", ss.clients().is_some());
}

fn main() {
    println!("=== ProtocolSettings Enum Examples ===\n");

    example_vless();
    example_shadowsocks();
    example_hysteria2();
    example_flowj();
    example_wireguard();
    example_deserialization();
    example_helper_methods();
}
