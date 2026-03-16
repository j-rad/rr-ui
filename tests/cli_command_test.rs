// tests/cli_command_test.rs
#[cfg(feature = "server")]
use rr_ui::adapters::uds_manager::{PanelSettings, SystemStatus, UdsRequest};

#[test]
#[cfg(feature = "server")]
fn test_uds_request_serialization() {
    // Test GetStatus request
    let request = UdsRequest::GetStatus;
    let serialized = bincode::serialize(&request).unwrap();
    let deserialized: UdsRequest = bincode::deserialize(&serialized).unwrap();

    match deserialized {
        UdsRequest::GetStatus => assert!(true),
        _ => panic!("Wrong request type"),
    }
}

#[test]
#[cfg(feature = "server")]
fn test_uds_response_serialization() {
    use rr_ui::adapters::uds_manager::UdsResponse;

    // Test Status response
    let status = SystemStatus {
        uptime_seconds: 3600,
        memory_mb: 512,
        cpu_percent: 25.5,
        active_connections: 10,
    };

    let response = UdsResponse::Status(status);
    let serialized = bincode::serialize(&response).unwrap();
    let deserialized: UdsResponse = bincode::deserialize(&serialized).unwrap();

    match deserialized {
        UdsResponse::Status(s) => {
            assert_eq!(s.uptime_seconds, 3600);
            assert_eq!(s.memory_mb, 512);
        }
        _ => panic!("Wrong response type"),
    }
}

#[test]
#[cfg(feature = "server")]
fn test_settings_response() {
    use rr_ui::adapters::uds_manager::UdsResponse;

    let settings = PanelSettings {
        port: 8081,
        username: "admin".to_string(),
        db_path: "/etc/rr-ui/rr-ui.db".to_string(),
        two_factor_enabled: false,
    };

    let response = UdsResponse::Settings(settings);
    let serialized = bincode::serialize(&response).unwrap();
    let deserialized: UdsResponse = bincode::deserialize(&serialized).unwrap();

    match deserialized {
        UdsResponse::Settings(s) => {
            assert_eq!(s.port, 8081);
            assert_eq!(s.username, "admin");
        }
        _ => panic!("Wrong response type"),
    }
}

#[test]
#[cfg(feature = "server")]
fn test_password_reset_request() {
    let request = UdsRequest::ResetPassword {
        new_password: "newpass123".to_string(),
    };

    let serialized = bincode::serialize(&request).unwrap();
    let deserialized: UdsRequest = bincode::deserialize(&serialized).unwrap();

    match deserialized {
        UdsRequest::ResetPassword { new_password } => {
            assert_eq!(new_password, "newpass123");
        }
        _ => panic!("Wrong request type"),
    }
}

#[test]
#[cfg(feature = "server")]
fn test_logs_request() {
    let request = UdsRequest::GetLogs { lines: 100 };

    let serialized = bincode::serialize(&request).unwrap();
    let deserialized: UdsRequest = bincode::deserialize(&serialized).unwrap();

    match deserialized {
        UdsRequest::GetLogs { lines } => {
            assert_eq!(lines, 100);
        }
        _ => panic!("Wrong request type"),
    }
}
