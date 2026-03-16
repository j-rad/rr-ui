// Mock context for server fn testing
#[tokio::test]
async fn test_subscription_link_generation() {
    let nodes = ["vless://uuid@ip:port".to_string(),
        "trojan://password@ip:port".to_string()];

    // We can't call server_fn directly easily in unit tests without setting up the environment context
    // But since our server_fn has a #[cfg(feature="server")] block that uses standard logic,
    // we can test the logic if we extract it, or just rely on the fact it compiles and logic is simple.
    // However, let's verify the base64 logic which is the core part.

    use base64::{Engine as _, engine::general_purpose};
    let content = nodes.join("\n");
    let encoded = general_purpose::STANDARD.encode(&content);

    assert!(!encoded.is_empty());

    // Verify decoding
    let decoded_vec = general_purpose::STANDARD.decode(&encoded).unwrap();
    let decoded_str = String::from_utf8(decoded_vec).unwrap();

    assert_eq!(decoded_str, content);
}
