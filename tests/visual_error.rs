//! Visual Error Test
//!
//! Mocks various gRPC and system errors to verify that the TacticalError
//! bridge and ToastItem component render correctly under different failures.

#[cfg(test)]
mod tests {
    use rr_ui::ui::error_bridge::{TacticalError, Level};

    #[test]
    fn test_mci_handshake_failure_suggestion() {
        let err = TacticalError::connection_failed("Handshake failed on MCI ASN: protocol mismatch");
        assert!(err.suggestion.contains("WS-CDN Carrier"), "Should suggest WS-CDN for MCI");
        assert_eq!(err.severity, Level::Error);
    }

    #[test]
    fn test_sni_filtering_suggestion() {
        let err = TacticalError::connection_failed("Remote closed connection during SNI verification");
        assert!(err.suggestion.contains("SNI Mutilation"), "Should suggest SNI Mutilation");
    }

    #[test]
    fn test_udp_throttling_suggestion() {
        let err = TacticalError::connection_failed("UDP packet loss > 90% detected");
        assert!(err.suggestion.contains("TCP-based transport"), "Should suggest TCP for UDP throttling");
    }

    #[test]
    fn test_service_unavailable_suggestion() {
        let err = TacticalError::connection_failed("connection refused");
        assert!(err.suggestion.contains("RustRay service is active"), "Should suggest checking service status");
    }
}
