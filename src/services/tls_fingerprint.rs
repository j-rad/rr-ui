// src/services/tls_fingerprint.rs
//! TLS Fingerprinting
//!
//! Randomizes TLS ClientHello to evade fingerprinting

use rand::Rng;
use serde::{Deserialize, Serialize};

/// TLS version
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TlsVersion {
    Tls12,
    Tls13,
}

impl TlsVersion {
    pub fn to_bytes(&self) -> [u8; 2] {
        match self {
            TlsVersion::Tls12 => [0x03, 0x03],
            TlsVersion::Tls13 => [0x03, 0x04],
        }
    }
}

/// TLS cipher suite
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CipherSuite {
    // TLS 1.3
    Aes128GcmSha256,
    Aes256GcmSha384,
    Chacha20Poly1305Sha256,
    // TLS 1.2
    EcdheRsaAes128GcmSha256,
    EcdheRsaAes256GcmSha384,
    EcdheRsaChacha20Poly1305,
}

impl CipherSuite {
    pub fn to_bytes(&self) -> [u8; 2] {
        match self {
            CipherSuite::Aes128GcmSha256 => [0x13, 0x01],
            CipherSuite::Aes256GcmSha384 => [0x13, 0x02],
            CipherSuite::Chacha20Poly1305Sha256 => [0x13, 0x03],
            CipherSuite::EcdheRsaAes128GcmSha256 => [0xc0, 0x2f],
            CipherSuite::EcdheRsaAes256GcmSha384 => [0xc0, 0x30],
            CipherSuite::EcdheRsaChacha20Poly1305 => [0xcc, 0xa8],
        }
    }
}

/// TLS extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TlsExtension {
    ServerName(String),
    SupportedGroups(Vec<u16>),
    SignatureAlgorithms(Vec<u16>),
    ApplicationLayerProtocolNegotiation(Vec<String>),
    SessionTicket,
    EncryptThenMac,
    ExtendedMasterSecret,
    SupportedVersions(Vec<TlsVersion>),
}

/// TLS fingerprint profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsFingerprint {
    pub version: TlsVersion,
    pub cipher_suites: Vec<CipherSuite>,
    pub extensions: Vec<TlsExtension>,
    pub compression_methods: Vec<u8>,
}

impl TlsFingerprint {
    /// Chrome-like fingerprint
    pub fn chrome() -> Self {
        Self {
            version: TlsVersion::Tls13,
            cipher_suites: vec![
                CipherSuite::Aes128GcmSha256,
                CipherSuite::Aes256GcmSha384,
                CipherSuite::Chacha20Poly1305Sha256,
                CipherSuite::EcdheRsaAes128GcmSha256,
                CipherSuite::EcdheRsaAes256GcmSha384,
            ],
            extensions: vec![
                TlsExtension::SupportedVersions(vec![TlsVersion::Tls13, TlsVersion::Tls12]),
                TlsExtension::SupportedGroups(vec![0x001d, 0x0017, 0x0018]),
                TlsExtension::SignatureAlgorithms(vec![0x0403, 0x0503, 0x0603]),
                TlsExtension::ApplicationLayerProtocolNegotiation(vec![
                    "h2".to_string(),
                    "http/1.1".to_string(),
                ]),
                TlsExtension::SessionTicket,
                TlsExtension::EncryptThenMac,
                TlsExtension::ExtendedMasterSecret,
            ],
            compression_methods: vec![0x00],
        }
    }

    /// Firefox-like fingerprint
    pub fn firefox() -> Self {
        Self {
            version: TlsVersion::Tls13,
            cipher_suites: vec![
                CipherSuite::Aes128GcmSha256,
                CipherSuite::Chacha20Poly1305Sha256,
                CipherSuite::Aes256GcmSha384,
                CipherSuite::EcdheRsaAes128GcmSha256,
            ],
            extensions: vec![
                TlsExtension::SupportedVersions(vec![TlsVersion::Tls13, TlsVersion::Tls12]),
                TlsExtension::SupportedGroups(vec![0x001d, 0x0017]),
                TlsExtension::SignatureAlgorithms(vec![0x0403, 0x0804, 0x0503]),
                TlsExtension::ApplicationLayerProtocolNegotiation(vec![
                    "h2".to_string(),
                    "http/1.1".to_string(),
                ]),
                TlsExtension::ExtendedMasterSecret,
            ],
            compression_methods: vec![0x00],
        }
    }

    /// Safari-like fingerprint
    pub fn safari() -> Self {
        Self {
            version: TlsVersion::Tls13,
            cipher_suites: vec![
                CipherSuite::Aes128GcmSha256,
                CipherSuite::Aes256GcmSha384,
                CipherSuite::Chacha20Poly1305Sha256,
            ],
            extensions: vec![
                TlsExtension::SupportedVersions(vec![TlsVersion::Tls13]),
                TlsExtension::SupportedGroups(vec![0x001d]),
                TlsExtension::SignatureAlgorithms(vec![0x0403, 0x0503]),
                TlsExtension::ApplicationLayerProtocolNegotiation(vec!["h2".to_string()]),
            ],
            compression_methods: vec![0x00],
        }
    }

    /// Random fingerprint (harder to detect)
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();

        let version = if rng.gen_bool(0.8) {
            TlsVersion::Tls13
        } else {
            TlsVersion::Tls12
        };

        let all_ciphers = vec![
            CipherSuite::Aes128GcmSha256,
            CipherSuite::Aes256GcmSha384,
            CipherSuite::Chacha20Poly1305Sha256,
            CipherSuite::EcdheRsaAes128GcmSha256,
            CipherSuite::EcdheRsaAes256GcmSha384,
            CipherSuite::EcdheRsaChacha20Poly1305,
        ];

        let num_ciphers = rng.gen_range(3..=6);
        let mut cipher_suites = Vec::new();
        for _ in 0..num_ciphers {
            let idx = rng.gen_range(0..all_ciphers.len());
            if !cipher_suites.contains(&all_ciphers[idx]) {
                cipher_suites.push(all_ciphers[idx]);
            }
        }

        Self {
            version,
            cipher_suites,
            extensions: vec![
                TlsExtension::SupportedVersions(vec![TlsVersion::Tls13, TlsVersion::Tls12]),
                TlsExtension::SupportedGroups(vec![0x001d, 0x0017, 0x0018]),
                TlsExtension::SignatureAlgorithms(vec![0x0403, 0x0503, 0x0603]),
            ],
            compression_methods: vec![0x00],
        }
    }
}

/// TLS fingerprint manager
pub struct FingerprintManager {
    profiles: Vec<TlsFingerprint>,
    rotation_interval_secs: u64,
    last_rotation: std::time::Instant,
    current_index: usize,
}

impl FingerprintManager {
    pub fn new(rotation_interval_secs: u64) -> Self {
        Self {
            profiles: vec![
                TlsFingerprint::chrome(),
                TlsFingerprint::firefox(),
                TlsFingerprint::safari(),
            ],
            rotation_interval_secs,
            last_rotation: std::time::Instant::now(),
            current_index: 0,
        }
    }

    /// Get current fingerprint
    pub fn current(&mut self) -> &TlsFingerprint {
        // Auto-rotate if interval passed
        if self.last_rotation.elapsed().as_secs() >= self.rotation_interval_secs {
            self.rotate();
        }

        &self.profiles[self.current_index]
    }

    /// Manually rotate fingerprint
    pub fn rotate(&mut self) {
        self.current_index = (self.current_index + 1) % self.profiles.len();
        self.last_rotation = std::time::Instant::now();
    }

    /// Add custom profile
    pub fn add_profile(&mut self, profile: TlsFingerprint) {
        self.profiles.push(profile);
    }

    /// Use random fingerprints
    pub fn enable_random_mode(&mut self) {
        self.profiles = vec![
            TlsFingerprint::random(),
            TlsFingerprint::random(),
            TlsFingerprint::random(),
        ];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tls_version_bytes() {
        assert_eq!(TlsVersion::Tls12.to_bytes(), [0x03, 0x03]);
        assert_eq!(TlsVersion::Tls13.to_bytes(), [0x03, 0x04]);
    }

    #[test]
    fn test_chrome_fingerprint() {
        let fp = TlsFingerprint::chrome();
        assert_eq!(fp.version, TlsVersion::Tls13);
        assert!(!fp.cipher_suites.is_empty());
        assert!(!fp.extensions.is_empty());
    }

    #[test]
    fn test_fingerprint_rotation() {
        let mut manager = FingerprintManager::new(3600);

        let fp1 = manager.current();
        assert_eq!(fp1.version, TlsVersion::Tls13);

        manager.rotate();
        let fp2 = manager.current();
        // Should be different profile after rotation
        assert_eq!(manager.current_index, 1);
    }

    #[test]
    fn test_random_fingerprint() {
        let fp1 = TlsFingerprint::random();
        let fp2 = TlsFingerprint::random();

        // Random fingerprints should potentially differ
        assert!(!fp1.cipher_suites.is_empty());
        assert!(!fp2.cipher_suites.is_empty());
    }
}
