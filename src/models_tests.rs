#[cfg(test)]
mod inbound_settings_tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_vless_serialization() {
        let vless = InboundSettings::Vless(VlessSettings {
            clients: vec![],
            decryption: Some("none".to_string()),
            fallbacks: None,
        });

        let json = serde_json::to_string(&vless).unwrap();
        assert!(json.contains("\"protocol\":\"vless\""));
        assert!(json.contains("\"decryption\":\"none\""));
    }

    #[test]
    fn test_shadowsocks_serialization() {
        let ss = InboundSettings::Shadowsocks(Shadowsocks2022Settings {
            method: "aes-256-gcm".to_string(),
            password: Some("test123".to_string()),
            key: None,
            network: Some("tcp,udp".to_string()),
            email: None,
            level: Some(0),
        });

        let json = serde_json::to_string(&ss).unwrap();
        assert!(json.contains("\"protocol\":\"shadowsocks\""));
        assert!(json.contains("\"method\":\"aes-256-gcm\""));
    }

    #[test]
    fn test_hysteria2_serialization() {
        let hy2 = InboundSettings::Hysteria2(Hysteria2Settings {
            up_mbps: Some(100),
            down_mbps: Some(200),
            password: Some("password".to_string()),
            obfuscation: None,
        });

        let json = serde_json::to_string(&hy2).unwrap();
        assert!(json.contains("\"protocol\":\"hysteria2\""));
        assert!(json.contains("\"upMbps\":100"));
        assert!(json.contains("\"downMbps\":200"));
    }

    #[test]
    fn test_flowj_mode_serialization() {
        let flowj = InboundSettings::FlowJ(FlowJSettings {
            clients: vec![],
            mode: FlowJMode::Reality,
            mqtt: None,
            reality: None,
            fec: None,
        });

        let json = serde_json::to_string(&flowj).unwrap();
        assert!(json.contains("\"protocol\":\"flowj\""));
        assert!(json.contains("\"mode\":\"reality\""));
    }

    #[test]
    fn test_wireguard_serialization() {
        let wg = InboundSettings::WireGuard(WireGuardSettings {
            secret_key: "test_secret_key".to_string(),
            peers: vec![],
            mtu: Some(1420),
            reserved: Some([0, 0, 0]),
        });

        let json = serde_json::to_string(&wg).unwrap();
        assert!(json.contains("\"protocol\":\"wireGuard\""));
        assert!(json.contains("\"secretKey\":\"test_secret_key\""));
        assert!(json.contains("\"mtu\":1420"));
    }

    #[test]
    fn test_clients_method() {
        let vless = InboundSettings::Vless(VlessSettings {
            clients: vec![Client::default()],
            decryption: None,
            fallbacks: None,
        });

        assert!(vless.clients().is_some());
        assert_eq!(vless.clients().unwrap().len(), 1);

        let ss = InboundSettings::Shadowsocks(Shadowsocks2022Settings::default());
        assert!(ss.clients().is_none());
    }

    #[test]
    fn test_protocol_name() {
        assert_eq!(
            InboundSettings::Vless(VlessSettings::default()).protocol_name(),
            "vless"
        );
        assert_eq!(
            InboundSettings::Vmess(VmessSettings::default()).protocol_name(),
            "vmess"
        );
        assert_eq!(
            InboundSettings::Trojan(TrojanSettings::default()).protocol_name(),
            "trojan"
        );
        assert_eq!(
            InboundSettings::Shadowsocks(Shadowsocks2022Settings::default()).protocol_name(),
            "shadowsocks"
        );
        assert_eq!(
            InboundSettings::Hysteria2(Hysteria2Settings::default()).protocol_name(),
            "hysteria2"
        );
        assert_eq!(
            InboundSettings::Tuic(TuicSettings::default()).protocol_name(),
            "tuic"
        );
        assert_eq!(
            InboundSettings::FlowJ(FlowJSettings::default()).protocol_name(),
            "flowj"
        );
        assert_eq!(
            InboundSettings::Naive(NaiveSettings::default()).protocol_name(),
            "naive"
        );
        assert_eq!(
            InboundSettings::WireGuard(WireGuardSettings::default()).protocol_name(),
            "wireguard"
        );
    }

    #[test]
    fn test_deserialization() {
        let json = r#"{
            "protocol": "vless",
            "settings": {
                "clients": [],
                "decryption": "none"
            }
        }"#;

        let settings: InboundSettings = serde_json::from_str(json).unwrap();
        match settings {
            InboundSettings::Vless(vless) => {
                assert_eq!(vless.decryption, Some("none".to_string()));
            }
            _ => panic!("Expected Vless variant"),
        }
    }
}
