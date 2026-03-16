use crate::models::Connection;
use actix_web::web::Bytes;
use log::error;
use serde_json;
use std::time::Duration;
use tokio::sync::broadcast::Sender;

pub struct SniffingService {
    tx: Sender<Bytes>,
}

impl SniffingService {
    pub fn new(tx: Sender<Bytes>) -> Self {
        Self { tx }
    }

    pub async fn run(self) {
        let mut interval = tokio::time::interval(Duration::from_secs(2)); // Tick every 2s
        let mut last_state: Vec<Connection> = Vec::new();

        loop {
            interval.tick().await;

            // Simulate fetching active connections from Core
            // In future: call orchestrator.get_active_connections()
            let current_state = self.mock_fetch_connections();

            // Optimization: Only broadcast if changed
            // Simple PartialEq check since Connection derives PartialEq
            if current_state != last_state {
                match serde_json::to_string(&current_state) {
                    Ok(json) => {
                        // Broadcast "sniffer:json" or just json?
                        // Middleware might need to distinguish stats vs sniffer if using same channel.
                        // For now we assume a dedicated channel or wrapped message.
                        // Let's wrap it in a simple structure or assume the UI handles it.
                        // Request says: "streams live connection objects". Use a wrapper key.
                        let msg = format!("{{\"liveConnections\":{}}}", json);
                        let _ = self.tx.send(Bytes::from(msg));
                        last_state = current_state;
                    }
                    Err(e) => {
                        error!("Failed to serialize connections: {}", e);
                    }
                }
            }
        }
    }

    fn mock_fetch_connections(&self) -> Vec<Connection> {
        // Generate simulated traffic
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let mut conns = Vec::new();
        // Vary list slightly based on time to test delta
        if now % 4000 < 2000 {
            conns.push(Connection {
                ip: "142.250.180.14".to_string(), // google
                domain: "google.com".to_string(),
                protocol: "tcp".to_string(),
                duration: 1200,
                latency: 45,
                id: None,
                inbound_tag: None,
                email: None,
                upload_speed: None,
                download_speed: None,
            });
        }
        conns.push(Connection {
            ip: "104.21.55.2".to_string(),
            domain: "cloudflare.com".to_string(),
            protocol: "tls".to_string(),
            duration: 5000 + (now % 1000), // Variable duration
            latency: 20,
            id: None,
            inbound_tag: None,
            email: None,
            upload_speed: None,
            download_speed: None,
        });

        conns
    }
}
