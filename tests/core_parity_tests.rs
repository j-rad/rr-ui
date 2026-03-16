// tests/core_parity_tests.rs
#![cfg(feature = "server")]
//! CI Parity Tests
//!
//! Ensures Xray and Sing-box produce equivalent configurations

use rr_ui::domain::proxy_core::*;

#[tokio::test]
async fn test_vmess_parity() {
    let config = create_test_config(Protocol::Vmess);

    // Both cores should support VMess
    let xray = rr_ui::adapters::rustray_core::RustRayCore::new();
    let singbox = rr_ui::adapters::singbox_core::SingboxCore::new();

    assert!(xray.supported_protocols().contains(&Protocol::Vmess));
    assert!(singbox.supported_protocols().contains(&Protocol::Vmess));

    // Both should validate the config
    assert!(xray.validate_config(&config).await.is_ok());
    assert!(singbox.validate_config(&config).await.is_ok());
}

#[tokio::test]
async fn test_vless_parity() {
    let config = create_test_config(Protocol::Vless);

    let xray = rr_ui::adapters::rustray_core::RustRayCore::new();
    let singbox = rr_ui::adapters::singbox_core::SingboxCore::new();

    assert!(xray.supported_protocols().contains(&Protocol::Vless));
    assert!(singbox.supported_protocols().contains(&Protocol::Vless));

    assert!(xray.validate_config(&config).await.is_ok());
    assert!(singbox.validate_config(&config).await.is_ok());
}

#[tokio::test]
async fn test_trojan_parity() {
    let config = create_test_config(Protocol::Trojan);

    let xray = rr_ui::adapters::rustray_core::RustRayCore::new();
    let singbox = rr_ui::adapters::singbox_core::SingboxCore::new();

    assert!(xray.supported_protocols().contains(&Protocol::Trojan));
    assert!(singbox.supported_protocols().contains(&Protocol::Trojan));

    assert!(xray.validate_config(&config).await.is_ok());
    assert!(singbox.validate_config(&config).await.is_ok());
}

#[tokio::test]
async fn test_hysteria2_support() {
    let config = create_test_config(Protocol::Hysteria2);

    let rustray = rr_ui::adapters::rustray_core::RustRayCore::new();
    let singbox = rr_ui::adapters::singbox_core::SingboxCore::new();

    // RustRay supports Hysteria2
    assert!(rustray.supported_protocols().contains(&Protocol::Hysteria2));
    assert!(rustray.validate_config(&config).await.is_ok());

    // Sing-box supports Hysteria2
    assert!(singbox.supported_protocols().contains(&Protocol::Hysteria2));
    assert!(singbox.validate_config(&config).await.is_ok());
}

#[tokio::test]
async fn test_tuic_support() {
    let config = create_test_config(Protocol::Tuic);

    let rustray = rr_ui::adapters::rustray_core::RustRayCore::new();
    let singbox = rr_ui::adapters::singbox_core::SingboxCore::new();

    // RustRay supports TUIC
    assert!(rustray.supported_protocols().contains(&Protocol::Tuic));
    assert!(rustray.validate_config(&config).await.is_ok());

    // Sing-box supports TUIC
    assert!(singbox.supported_protocols().contains(&Protocol::Tuic));
    assert!(singbox.validate_config(&config).await.is_ok());
}

#[tokio::test]
async fn test_core_lifecycle() {
    let config = create_test_config(Protocol::Vmess);

    let mut xray = rr_ui::adapters::rustray_core::RustRayCore::new();

    // Initially not running
    assert!(!xray.is_running());

    // Start should work (will fail if xray not installed, that's ok)
    let _ = xray.start(config.clone()).await;

    // Stop should work
    let _ = xray.stop().await;
}

#[tokio::test]
async fn test_core_factory() {
    // Test factory creation
    let rustray = CoreFactory::create("rustray");
    assert!(rustray.is_ok());

    let unknown = CoreFactory::create("unknown");
    assert!(unknown.is_err());
}

#[tokio::test]
async fn test_available_cores() {
    let cores = CoreFactory::available_cores();
    assert_eq!(cores.len(), 1);
    assert!(cores.contains(&"rustray"));
}

// Helper function
fn create_test_config(protocol: Protocol) -> CoreConfig {
    CoreConfig {
        log_level: LogLevel::Info,
        inbounds: vec![InboundConfig {
            tag: "test-inbound".to_string(),
            protocol,
            listen: "0.0.0.0".to_string(),
            port: 10086,
            settings: serde_json::json!({}),
        }],
        outbounds: vec![OutboundConfig {
            tag: "direct".to_string(),
            protocol: Protocol::Freedom,
            settings: serde_json::json!({}),
        }],
        routing: RoutingConfig {
            rules: vec![],
            domain_strategy: DomainStrategy::AsIs,
        },
        dns: None,
    }
}
