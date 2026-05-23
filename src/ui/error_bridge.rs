//! Tactical Error Bridge
//!
//! Maps system, gRPC, and transport errors to user-friendly tactical suggestions.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Level {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TacticalMessage {
    pub title: String,
    pub message: String,
    pub suggestion: String,
    pub severity: Level,
}

impl TacticalMessage {
    pub fn new(title: impl Into<String>, message: impl Into<String>, suggestion: impl Into<String>, severity: Level) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            suggestion: suggestion.into(),
            severity,
        }
    }

    /// Generic connection failed error
    pub fn connection_failed(details: &str) -> Self {
        let details_lower = details.to_lowercase();
        
        let (title, suggestion, severity) = if details_lower.contains("unavailable") {
            if details_lower.contains("mci") {
                ("Core Unreachable (MCI)", "MCI UDP Throttling confirmed. Action: Pivot to FakeTCP or CDN-Loop.", Level::Critical)
            } else {
                ("Core Unreachable", "Check if RustRay service is active and API port (10085) is open.", Level::Critical)
            }
        } else if details_lower.contains("byte 1") || details_lower.contains("handshake failed at byte 1") {
            ("Handshake Blocked", "ServerHello Blocked. Action: Trigger 1-byte Fragmentation.", Level::Error)
        } else if details_lower.contains("udp") || details_lower.contains("quic") {
            ("UDP Throttling", "UDP packets are being dropped. Switch to TCP-based transport (WebSocket/gRPC) or use Hysteria2 with Obfuscation.", Level::Error)
        } else if details_lower.contains("timeout") || details_lower.contains("deadline") {
            ("Tactical Timeout", "Network latency too high or core overloaded. Try switching to a lower-latency transport.", Level::Warning)
        } else if details_lower.contains("handshake") && (details_lower.contains("irancell") || details_lower.contains("zaman")) {
             ("ASN Blockade Detected", "Handshake failure detected on restrictive ASN. Enable WS-CDN Carrier or Switch to Fragmented TLS.", Level::Error)
        } else if details_lower.contains("sni") || details_lower.contains("host mismatch") {
            ("SNI Filtering", "SNI Filtering detected. Use a different decoy domain or Enable SNI Mutilation (Fragmented TLS).", Level::Error)
        } else if details_lower.contains("permission") || details_lower.contains("access denied") {
            ("Security Violation", "Verify API tokens and permission levels. Firewall might be blocking the API port.", Level::Critical)
        } else if details_lower.contains("database") || details_lower.contains("surreal") {
            ("Database Error", "The persistence layer is failing. Check disk space or SurrealDB status.", Level::Error)
        } else {
            ("Transport Error", "Verify network connectivity and proxy settings.", Level::Error)
        };

        Self::new(title, details, suggestion, severity)
    }
}

#[cfg(feature = "server")]
impl From<tonic::Status> for TacticalMessage {
    fn from(status: tonic::Status) -> Self {
        let details = status.message();
        let code = status.code();

        match code {
            tonic::Code::Unavailable => {
                let details_lower = details.to_lowercase();
                if details_lower.contains("mci") {
                    Self::new(
                        "Core Unreachable (MCI)",
                        details,
                        "MCI UDP Throttling confirmed. Action: Pivot to FakeTCP or CDN-Loop.",
                        Level::Critical
                    )
                } else {
                    Self::new(
                        "Service Unavailable",
                        details,
                        "Check if RustRay service is active and API port (10085) is open.",
                        Level::Critical
                    )
                }
            },
            tonic::Code::DeadlineExceeded => Self::new(
                "Request Timeout",
                details,
                "The core took too long to respond. Check system load.",
                Level::Warning
            ),
            tonic::Code::PermissionDenied | tonic::Code::Unauthenticated => Self::new(
                "Access Denied",
                details,
                "Verify API tokens and permission levels.",
                Level::Error
            ),
            _ => {
                if details.to_lowercase().contains("byte 1") {
                    Self::new(
                        "Handshake Blocked",
                        details,
                        "ServerHello Blocked. Action: Trigger 1-byte Fragmentation.",
                        Level::Error
                    )
                } else {
                    Self::connection_failed(details)
                }
            }
        }
    }
}

// Fallback for non-tonic errors
impl From<String> for TacticalMessage {
    fn from(msg: String) -> Self {
        Self::connection_failed(&msg)
    }
}

impl From<&str> for TacticalMessage {
    fn from(msg: &str) -> Self {
        Self::connection_failed(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tactical_logic_unavailable() {
        let err = TacticalMessage::connection_failed("connection refused");
        assert_eq!(err.suggestion, "Check if RustRay service is active and API port (10085) is open.");
    }

    #[test]
    fn test_tactical_logic_asn_failure() {
        let err = TacticalMessage::connection_failed("handshake failure on irancell node");
        assert_eq!(err.suggestion, "Handshake failure detected on restrictive ASN. Enable WS-CDN Carrier or Switch to Fragmented TLS.");
    }

    #[test]
    fn test_tactical_logic_sni_filtering() {
        let err = TacticalMessage::connection_failed("SNI mismatch or filtered");
        assert_eq!(err.title, "SNI Filtering");
        assert_eq!(err.suggestion, "SNI Filtering detected. Use a different decoy domain or Enable SNI Mutilation (Fragmented TLS).");
    }

    #[test]
    fn test_tactical_logic_udp_throttling() {
        let err = TacticalMessage::connection_failed("UDP buffer timeout or dropped");
        assert_eq!(err.title, "UDP Throttling");
        assert_eq!(err.suggestion, "UDP packets are being dropped. Switch to TCP-based transport (WebSocket/gRPC) or use Hysteria2 with Obfuscation.");
    }

    #[test]
    fn test_tactical_logic_security_violation() {
        let err = TacticalMessage::connection_failed("permission denied to access 127.0.0.1:10085");
        assert_eq!(err.title, "Security Violation");
        assert_eq!(err.severity, Level::Critical);
    }
}
