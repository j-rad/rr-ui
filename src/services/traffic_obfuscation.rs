// src/services/traffic_obfuscation.rs
//! Traffic Obfuscation
//!
//! Disguises proxy traffic as normal HTTPS/WebSocket traffic

use rand::Rng;
use serde::{Deserialize, Serialize};

/// Obfuscation method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObfuscationMethod {
    /// Plain traffic (no obfuscation)
    None,
    /// HTTP/2 masquerading
    Http2,
    /// WebSocket masquerading
    WebSocket,
    /// TLS-in-TLS (double encryption)
    TlsInTls,
    /// Custom padding
    Padding { min_bytes: usize, max_bytes: usize },
}

/// Traffic obfuscator
pub struct TrafficObfuscator {
    method: ObfuscationMethod,
    domain_fronting_enabled: bool,
    fronting_domain: Option<String>,
}

impl TrafficObfuscator {
    pub fn new(method: ObfuscationMethod) -> Self {
        Self {
            method,
            domain_fronting_enabled: false,
            fronting_domain: None,
        }
    }

    /// Enable domain fronting
    pub fn enable_domain_fronting(&mut self, fronting_domain: String) {
        self.domain_fronting_enabled = true;
        self.fronting_domain = Some(fronting_domain);
    }

    /// Obfuscate outgoing data
    pub fn obfuscate(&self, data: &[u8]) -> Vec<u8> {
        match &self.method {
            ObfuscationMethod::None => data.to_vec(),
            ObfuscationMethod::Http2 => self.wrap_http2(data),
            ObfuscationMethod::WebSocket => self.wrap_websocket(data),
            ObfuscationMethod::TlsInTls => self.wrap_tls(data),
            ObfuscationMethod::Padding {
                min_bytes,
                max_bytes,
            } => self.add_padding(data, *min_bytes, *max_bytes),
        }
    }

    /// Deobfuscate incoming data
    pub fn deobfuscate(&self, data: &[u8]) -> Vec<u8> {
        match &self.method {
            ObfuscationMethod::None => data.to_vec(),
            ObfuscationMethod::Http2 => self.unwrap_http2(data),
            ObfuscationMethod::WebSocket => self.unwrap_websocket(data),
            ObfuscationMethod::TlsInTls => self.unwrap_tls(data),
            ObfuscationMethod::Padding { .. } => self.remove_padding(data),
        }
    }

    fn wrap_http2(&self, data: &[u8]) -> Vec<u8> {
        // HTTP/2 frame format:
        // +-----------------------------------------------+
        // |                 Length (24)                   |
        // +---------------+---------------+---------------+
        // |   Type (8)    |   Flags (8)   |
        // +-+-------------+---------------+-------------------------------+
        // |R|                 Stream Identifier (31)                      |
        // +=+=============================================================+
        // |                   Frame Payload (0...)                      ...
        // +---------------------------------------------------------------+

        let mut frame = Vec::new();

        // Length (24 bits)
        let len = data.len() as u32;
        frame.push(((len >> 16) & 0xFF) as u8);
        frame.push(((len >> 8) & 0xFF) as u8);
        frame.push((len & 0xFF) as u8);

        // Type: DATA (0x0)
        frame.push(0x00);

        // Flags: END_STREAM (0x1)
        frame.push(0x01);

        // Stream ID (31 bits) - use stream 1
        frame.extend_from_slice(&[0x00, 0x00, 0x00, 0x01]);

        // Payload
        frame.extend_from_slice(data);

        frame
    }

    fn unwrap_http2(&self, data: &[u8]) -> Vec<u8> {
        if data.len() < 9 {
            return data.to_vec();
        }

        // Skip 9-byte header
        data[9..].to_vec()
    }

    fn wrap_websocket(&self, data: &[u8]) -> Vec<u8> {
        // WebSocket frame format:
        //  0                   1                   2                   3
        //  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
        // +-+-+-+-+-------+-+-------------+-------------------------------+
        // |F|R|R|R| opcode|M| Payload len |    Extended payload length    |
        // |I|S|S|S|  (4)  |A|     (7)     |             (16/64)           |
        // |N|V|V|V|       |S|             |   (if payload len==126/127)   |
        // | |1|2|3|       |K|             |                               |
        // +-+-+-+-+-------+-+-------------+ - - - - - - - - - - - - - - - +

        let mut frame = Vec::new();

        // FIN=1, RSV=0, opcode=2 (binary)
        frame.push(0x82);

        // MASK=1, payload length
        let len = data.len();
        if len < 126 {
            frame.push(0x80 | (len as u8));
        } else if len < 65536 {
            frame.push(0x80 | 126);
            frame.push(((len >> 8) & 0xFF) as u8);
            frame.push((len & 0xFF) as u8);
        } else {
            frame.push(0x80 | 127);
            frame.extend_from_slice(&(len as u64).to_be_bytes());
        }

        // Masking key (random)
        let mut rng = rand::thread_rng();
        let mask: [u8; 4] = [rng.r#gen(), rng.r#gen(), rng.r#gen(), rng.r#gen()];
        frame.extend_from_slice(&mask);

        // Masked payload
        for (i, byte) in data.iter().enumerate() {
            frame.push(byte ^ mask[i % 4]);
        }

        frame
    }

    fn unwrap_websocket(&self, data: &[u8]) -> Vec<u8> {
        if data.len() < 6 {
            return data.to_vec();
        }

        let payload_len = (data[1] & 0x7F) as usize;
        let mut offset = 2;

        let actual_len = if payload_len == 126 {
            if data.len() < 4 {
                return data.to_vec();
            }
            offset = 4;
            ((data[2] as usize) << 8) | (data[3] as usize)
        } else if payload_len == 127 {
            if data.len() < 10 {
                return data.to_vec();
            }
            offset = 10;
            u64::from_be_bytes([
                data[2], data[3], data[4], data[5], data[6], data[7], data[8], data[9],
            ]) as usize
        } else {
            payload_len
        };

        // Extract mask
        if data.len() < offset + 4 {
            return data.to_vec();
        }

        let mask = [
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ];
        offset += 4;

        // Unmask payload
        let mut payload = Vec::new();
        for i in 0..actual_len {
            if offset + i >= data.len() {
                break;
            }
            payload.push(data[offset + i] ^ mask[i % 4]);
        }

        payload
    }

    fn wrap_tls(&self, data: &[u8]) -> Vec<u8> {
        // Simplified TLS record layer
        let mut record = Vec::new();

        // Content type: Application Data (23)
        record.push(23);

        // Version: TLS 1.2
        record.extend_from_slice(&[0x03, 0x03]);

        // Length
        let len = data.len() as u16;
        record.extend_from_slice(&len.to_be_bytes());

        // Data
        record.extend_from_slice(data);

        record
    }

    fn unwrap_tls(&self, data: &[u8]) -> Vec<u8> {
        if data.len() < 5 {
            return data.to_vec();
        }

        // Skip 5-byte TLS record header
        data[5..].to_vec()
    }

    fn add_padding(&self, data: &[u8], min_bytes: usize, max_bytes: usize) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        let padding_len = rng.gen_range(min_bytes..=max_bytes);

        let mut padded = Vec::with_capacity(data.len() + padding_len + 2);

        // Original data length (2 bytes)
        let data_len = data.len() as u16;
        padded.extend_from_slice(&data_len.to_be_bytes());

        // Original data
        padded.extend_from_slice(data);

        // Random padding
        for _ in 0..padding_len {
            padded.push(rng.r#gen());
        }

        padded
    }

    fn remove_padding(&self, data: &[u8]) -> Vec<u8> {
        if data.len() < 2 {
            return data.to_vec();
        }

        let data_len = u16::from_be_bytes([data[0], data[1]]) as usize;

        if data.len() < 2 + data_len {
            return data.to_vec();
        }

        data[2..2 + data_len].to_vec()
    }

    /// Get SNI (Server Name Indication) for domain fronting
    pub fn get_sni(&self) -> Option<&str> {
        if self.domain_fronting_enabled {
            self.fronting_domain.as_deref()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http2_wrapping() {
        let obfuscator = TrafficObfuscator::new(ObfuscationMethod::Http2);
        let data = b"Hello, World!";

        let wrapped = obfuscator.obfuscate(data);
        assert!(wrapped.len() > data.len());

        let unwrapped = obfuscator.deobfuscate(&wrapped);
        assert_eq!(unwrapped, data);
    }

    #[test]
    fn test_websocket_wrapping() {
        let obfuscator = TrafficObfuscator::new(ObfuscationMethod::WebSocket);
        let data = b"Test data";

        let wrapped = obfuscator.obfuscate(data);
        assert!(wrapped.len() > data.len());

        let unwrapped = obfuscator.deobfuscate(&wrapped);
        assert_eq!(unwrapped, data);
    }

    #[test]
    fn test_padding() {
        let obfuscator = TrafficObfuscator::new(ObfuscationMethod::Padding {
            min_bytes: 10,
            max_bytes: 50,
        });

        let data = b"Short";
        let padded = obfuscator.obfuscate(data);

        assert!(padded.len() >= data.len() + 10);
        assert!(padded.len() <= data.len() + 50 + 2);

        let unpadded = obfuscator.deobfuscate(&padded);
        assert_eq!(unpadded, data);
    }

    #[test]
    fn test_domain_fronting() {
        let mut obfuscator = TrafficObfuscator::new(ObfuscationMethod::None);
        obfuscator.enable_domain_fronting("cdn.cloudflare.com".to_string());

        assert_eq!(obfuscator.get_sni(), Some("cdn.cloudflare.com"));
    }

    #[test]
    fn test_tls_wrapping() {
        let obfuscator = TrafficObfuscator::new(ObfuscationMethod::TlsInTls);
        let data = b"Encrypted payload";

        let wrapped = obfuscator.obfuscate(data);
        assert_eq!(wrapped[0], 23); // Application Data

        let unwrapped = obfuscator.deobfuscate(&wrapped);
        assert_eq!(unwrapped, data);
    }
}
