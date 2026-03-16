use rr_ui::domain::models::*;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_vless_parity_with_extra_fields() {
    // Simulate a JSON from a newer Xray version that has an unknown field
    let json = json!({
        "clients": [],
        "decryption": "none",
        "fallbacks": null,
        "new_experimental_field": "some_value"
    });

    // Deserializing into VlessSettings should capture the unknown field in `extra`
    let settings: VlessSettings = serde_json::from_value(json).unwrap();

    // Verify standard fields
    assert_eq!(settings.decryption.as_deref(), Some("none"));

    // Verify extra field
    assert!(settings.extra.contains_key("new_experimental_field"));
    assert_eq!(settings.extra["new_experimental_field"], "some_value");

    // Serializing back should verify the extra field is flattened
    let output_json = serde_json::to_value(&settings).unwrap();
    assert_eq!(output_json["new_experimental_field"], "some_value");
}

#[test]
fn test_reality_pqc_serialization() {
    let mut extra = HashMap::new();
    extra.insert("custom_field".to_string(), json!(123));

    let settings = RealitySettings {
        show: true,
        dest: "example.com:443".into(),
        xver: 0,
        server_names: vec!["example.com".into()],
        private_key: "key".into(),
        short_ids: vec![],
        fingerprint: "chrome".into(),
        min_client_ver: None,
        max_client_ver: None,
        pqc_matrix: Some(PqcMatrix::Kyber768),
        stealth_handshake: false,
        extra,
    };

    let json = serde_json::to_value(&settings).unwrap();

    assert_eq!(json["pqcCipher"], json!("kyber768")); // or check exact serialization
    assert_eq!(json["custom_field"], 123);
}

#[test]
fn test_flow_type_deserialization_on_client() {
    // Note: Client struct currently uses Option<String> for flow,
    // so we test generic deserialization compatibility manually or
    // waiting for Client struct update.
    // For now, let's test FlowType enum itself

    let json = json!("xtls-rprx-vision");
    let flow: FlowType = serde_json::from_value(json).unwrap();
    assert_eq!(flow, FlowType::XtlsRprxVision);

    let json_unknown = json!("unknown-flow");
    let flow_unknown: FlowType = serde_json::from_value(json_unknown).unwrap();
    match flow_unknown {
        FlowType::Other(s) => assert_eq!(s, "unknown-flow"),
        _ => panic!("Should be Other variant"),
    }
}
