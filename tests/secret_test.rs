use rr_ui::models::{AllSetting, Client, LoginPayload};

#[test]
fn test_secret_serialization() {
    // 1. Test LoginPayload deserialization (from API request)
    let json = r#"{"username": "admin", "password": "mysecretpassword"}"#;
    let payload: LoginPayload = serde_json::from_str(json).unwrap();
    assert_eq!(payload.username, "admin");
    assert_eq!(payload.password, "mysecretpassword");

    // Debug would be redacted if password was SecretString
    // let debug_str = format!("{:?}", payload);
    // assert!(!debug_str.contains("mysecretpassword"));
    // assert!(debug_str.contains("[REDACTED]"));

    // 2. Test AllSetting serialization (to DB/API)
    // We expect it to be EXPOSED because we use custom serializer,
    // assuming we want to save it to DB or send to authorized API client.
    let settings = AllSetting {
        two_factor_secret: Some("mfa_secret_key".to_string()),
        ..Default::default()
    };

    let serialized = serde_json::to_string(&settings).unwrap();
    assert!(serialized.contains("mfa_secret_key")); // Must be exposed for storage/API

    // 3. Test Client serialization
    let client = Client {
        password: Some("trojan_pass".to_string()),
        ..Default::default()
    };
    let serialized_client = serde_json::to_string(&client).unwrap();
    assert!(serialized_client.contains("trojan_pass"));
}
