use crate::domain::models::*;
use serde::Serialize;

// Helper to verify round-trip serialization
fn verify_serialization<
    T: Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
>(
    original: &T,
) {
    let serialized = serde_json::to_string(original).expect("Failed to serialize");
    let deserialized: T = serde_json::from_str(&serialized).expect("Failed to deserialize");
    assert_eq!(original, &deserialized);
}

#[test]
fn test_inbound_protocol_enum_serialization() {
    assert_eq!(
        serde_json::to_string(&InboundProtocol::Vless).unwrap(),
        "\"vless\""
    );
    assert_eq!(
        serde_json::to_string(&InboundProtocol::Vmess).unwrap(),
        "\"vmess\""
    );
    assert_eq!(
        serde_json::to_string(&InboundProtocol::FlowJ).unwrap(),
        "\"flowj\""
    );
    assert_eq!(
        serde_json::to_string(&InboundProtocol::Dokodemo).unwrap(),
        "\"dokodemo-door\""
    );

    let p: InboundProtocol = serde_json::from_str("\"flowj\"").unwrap();
    assert_eq!(p, InboundProtocol::FlowJ);
}

#[test]
fn test_user_role_serialization() {
    assert_eq!(
        serde_json::to_string(&UserRole::Admin).unwrap(),
        "\"admin\""
    );
    assert_eq!(
        serde_json::to_string(&UserRole::Reseller).unwrap(),
        "\"reseller\""
    );

    let role: UserRole = serde_json::from_str("\"reseller\"").unwrap();
    assert_eq!(role, UserRole::Reseller);
}

#[test]
fn test_client_serialization_strict_types() {
    let client = Client {
        id: Some("uuid-1234".to_string()),
        email: Some("user@example.com".to_string()),
        total_flow_limit: 1024 * 1024 * 1024, // 1GB
        limit_ip: Some(3),
        enable: true,
        created_by: Some("42".to_string()),
        ..Default::default()
    };

    // Verify alias works (we can't easily verify serialization output field name without snapshot,
    // but we can check if it creates the value we expect)
    let json = serde_json::to_value(&client).unwrap();
    // Default serialization uses field name 'totalFlowLimit' (camelCase)
    assert_eq!(json["totalFlowLimit"], 1073741824);
    assert_eq!(json["created_by"], "42");
}

#[test]
fn test_legacy_client_deserialization() {
    // Test parsing 3x-ui style JSON with legacy fields
    let json_str = r#"{
        "id": "abc-123",
        "email": "test@test.com",
        "total_gb": 500000,
        "ip_limit": 5,
        "enable": true
    }"#;

    let client: Client = serde_json::from_str(json_str).expect("Failed to parse legacy client");
    assert_eq!(client.total_flow_limit, 500000);
    assert_eq!(client.limit_ip, Some(5));
}

#[test]
fn test_pqc_matrix_variants() {
    let s = RealitySettings {
        pqc_matrix: Some(PqcMatrix::Dilithium2),
        ..Default::default()
    };
    let json = serde_json::to_string(&s).unwrap();
    assert!(json.contains("\"pqcCipher\":\"dilithium2\"")); // Renamed field in JSON to 'pqcCipher' for compat? 
    // Wait, in my models.rs I used #[serde(rename="pqcCipher")] on the field?
    // Let's check my replacement.
    // Chunk 5: `#[serde(skip_serializing_if = "Option::is_none", rename="pqcCipher")] pub pqc_matrix: Option<PqcMatrix>`
    // Yes, I did. So it should serialize to "pqcCipher".
}

#[test]
fn test_inbound_full_serialization() {
    let inbound = Inbound {
        protocol: InboundProtocol::Vless,
        port: 443,
        tag: Cow::Borrowed("vless-in"),
        settings: ProtocolSettings::Vless(VlessSettings {
            clients: vec![Client {
                id: Some("uuid".into()),
                email: Some("test".into()),
                ..Default::default()
            }],
            decryption: Some(Cow::Borrowed("none")),
            ..Default::default()
        }),
        stream_settings: StreamSettings {
            network: Cow::Borrowed("ws"),
            security: Cow::Borrowed("tls"),
            tls_settings: Some(TlsSettings {
                server_name: Cow::Borrowed("example.com"),
                ..Default::default()
            }),
            ..Default::default()
        },
        ..Default::default()
    };

    verify_serialization(&inbound);
}

#[test]
fn test_dokodemo_serialization() {
    let s = DokodemoSettings {
        address: "127.0.0.1".into(),
        port: 80,
        network: "tcp,udp".into(),
        follow_redirect: true,
    };
    verify_serialization(&s);
}

#[test]
fn test_socks_serialization() {
    let s = SocksSettings {
        auth: "password".into(),
        accounts: vec![Client {
            email: Some("user".into()),
            password: Some("pass".into()),
            ..Default::default()
        }],
        udp: true,
        bind_address: "127.0.0.1".into(),
    };
    verify_serialization(&s);
}
