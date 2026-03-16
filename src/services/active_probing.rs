// src/services/active_probing.rs
//! Active Probing System
//!
//! Async TCP/UDP RTT tests for node health monitoring

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::net::{TcpSocket, TcpStream, UdpSocket};
use tokio::time::timeout;

/// Probe result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResult {
    pub node_id: String,
    pub timestamp: i64,
    pub protocol: ProbeProtocol,
    pub rtt_ms: Option<f64>,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProbeProtocol {
    Tcp,
    Udp,
    Icmp,
}

/// Node health statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeHealthStats {
    pub node_id: String,
    pub avg_rtt_ms: f64,
    pub min_rtt_ms: f64,
    pub max_rtt_ms: f64,
    pub success_rate: f32,
    pub consecutive_failures: u32,
    pub last_success: i64,
    pub last_failure: i64,
    pub health_score: f32, // 0.0 - 1.0
}

impl NodeHealthStats {
    pub fn new(node_id: String) -> Self {
        Self {
            node_id,
            avg_rtt_ms: 0.0,
            min_rtt_ms: f64::MAX,
            max_rtt_ms: 0.0,
            success_rate: 1.0,
            consecutive_failures: 0,
            last_success: chrono::Utc::now().timestamp(),
            last_failure: 0,
            health_score: 1.0,
        }
    }

    /// Update stats with new probe result
    pub fn update(&mut self, result: &ProbeResult) {
        let now = chrono::Utc::now().timestamp();

        if result.success {
            if let Some(rtt) = result.rtt_ms {
                // Update RTT stats
                if self.avg_rtt_ms == 0.0 {
                    self.avg_rtt_ms = rtt;
                } else {
                    // Exponential moving average
                    self.avg_rtt_ms = self.avg_rtt_ms * 0.7 + rtt * 0.3;
                }
                self.min_rtt_ms = self.min_rtt_ms.min(rtt);
                self.max_rtt_ms = self.max_rtt_ms.max(rtt);
            }

            self.consecutive_failures = 0;
            self.last_success = now;
            self.success_rate = (self.success_rate * 0.95 + 0.05).min(1.0);
        } else {
            self.consecutive_failures += 1;
            self.last_failure = now;
            self.success_rate = (self.success_rate * 0.95).max(0.0);
        }

        // Calculate health score (0.0 - 1.0)
        self.health_score = self.calculate_health_score();
    }

    fn calculate_health_score(&self) -> f32 {
        let mut score = self.success_rate;

        // Penalty for consecutive failures
        if self.consecutive_failures > 0 {
            score *= 0.9_f32.powi(self.consecutive_failures as i32);
        }

        // Penalty for high RTT
        if self.avg_rtt_ms > 500.0 {
            score *= 0.8;
        } else if self.avg_rtt_ms > 200.0 {
            score *= 0.9;
        }

        score.max(0.0).min(1.0)
    }

    pub fn is_healthy(&self) -> bool {
        self.health_score >= 0.7 && self.consecutive_failures < 3
    }
}

/// Active prober
pub struct ActiveProber {
    probe_timeout: Duration,
    probe_interval: Duration,
}

impl ActiveProber {
    pub fn new(timeout_ms: u64, interval_secs: u64) -> Self {
        Self {
            probe_timeout: Duration::from_millis(timeout_ms),
            probe_interval: Duration::from_secs(interval_secs),
        }
    }

    pub fn probe_interval(&self) -> Duration {
        self.probe_interval
    }

    pub fn probe_interval_secs(&self) -> u64 {
        self.probe_interval.as_secs()
    }

    /// Probe TCP endpoint
    pub async fn probe_tcp(&self, node_id: &str, address: &str, port: u16, source_ip: Option<String>) -> ProbeResult {
        let start = Instant::now();
        let target_str = format!("{}:{}", address, port);

        // Resolve address
        let addr = match tokio::net::lookup_host(&target_str).await {
            Ok(mut addrs) => match addrs.next() {
                Some(a) => a,
                None => return self.fail_result(node_id, "Could not resolve host"),
            },
            Err(e) => return self.fail_result(node_id, &e.to_string()),
        };

        let socket = match addr {
            SocketAddr::V4(_) => TcpSocket::new_v4(),
            SocketAddr::V6(_) => TcpSocket::new_v6(),
        };

        let socket = match socket {
            Ok(s) => s,
            Err(e) => return self.fail_result(node_id, &e.to_string()),
        };

        // Bind to source IP if provided
        if let Some(src) = source_ip {
            if let Ok(src_addr) = src.parse::<SocketAddr>() {
                if let Err(e) = socket.bind(src_addr) {
                    return self.fail_result(node_id, &format!("Failed to bind source IP: {}", e));
                }
            }
        }

        let result = timeout(self.probe_timeout, socket.connect(addr)).await;

        match result {
            Ok(Ok(_stream)) => {
                let rtt_ms = start.elapsed().as_secs_f64() * 1000.0;
                ProbeResult {
                    node_id: node_id.to_string(),
                    timestamp: chrono::Utc::now().timestamp(),
                    protocol: ProbeProtocol::Tcp,
                    rtt_ms: Some(rtt_ms),
                    success: true,
                    error: None,
                }
            }
            Ok(Err(e)) => self.fail_result(node_id, &e.to_string()),
            Err(_) => self.fail_result(node_id, "Timeout"),
        }
    }

    fn fail_result(&self, node_id: &str, error: &str) -> ProbeResult {
        ProbeResult {
            node_id: node_id.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            protocol: ProbeProtocol::Tcp,
            rtt_ms: None,
            success: false,
            error: Some(error.to_string()),
        }
    }

    /// Probe UDP endpoint
    pub async fn probe_udp(&self, node_id: &str, address: &str, port: u16) -> ProbeResult {
        let start = Instant::now();
        let target = format!("{}:{}", address, port);

        let result = timeout(self.probe_timeout, async {
            let socket = UdpSocket::bind("0.0.0.0:0").await?;
            socket.connect(&target).await?;

            // Send probe packet
            let probe_data = b"PROBE";
            socket.send(probe_data).await?;

            // Try to receive response
            let mut buf = [0u8; 1024];
            socket.recv(&mut buf).await?;

            Ok::<(), std::io::Error>(())
        })
        .await;

        match result {
            Ok(Ok(())) => {
                let rtt_ms = start.elapsed().as_secs_f64() * 1000.0;
                ProbeResult {
                    node_id: node_id.to_string(),
                    timestamp: chrono::Utc::now().timestamp(),
                    protocol: ProbeProtocol::Udp,
                    rtt_ms: Some(rtt_ms),
                    success: true,
                    error: None,
                }
            }
            Ok(Err(e)) => ProbeResult {
                node_id: node_id.to_string(),
                timestamp: chrono::Utc::now().timestamp(),
                protocol: ProbeProtocol::Udp,
                rtt_ms: None,
                success: false,
                error: Some(e.to_string()),
            },
            Err(_) => ProbeResult {
                node_id: node_id.to_string(),
                timestamp: chrono::Utc::now().timestamp(),
                protocol: ProbeProtocol::Udp,
                rtt_ms: None,
                success: false,
                error: Some("Timeout".to_string()),
            },
        }
    }

    /// Start continuous probing loop
    pub async fn start_probing<F>(
        &self,
        node_id: String,
        address: String,
        port: u16,
        protocol: ProbeProtocol,
        mut callback: F,
    ) where
        F: FnMut(ProbeResult) + Send + 'static,
    {
        let mut interval = tokio::time::interval(self.probe_interval);

        loop {
            interval.tick().await;

            let result = match protocol {
                ProbeProtocol::Tcp => self.probe_tcp(&node_id, &address, port, None).await,
                ProbeProtocol::Udp => self.probe_udp(&node_id, &address, port).await,
                ProbeProtocol::Icmp => {
                    // ICMP requires raw sockets (root privileges)
                    // Fallback to TCP for now
                    self.probe_tcp(&node_id, &address, port, None).await
                }
            };

            callback(result);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_stats_update() {
        let mut stats = NodeHealthStats::new("node1".to_string());

        let success_result = ProbeResult {
            node_id: "node1".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            protocol: ProbeProtocol::Tcp,
            rtt_ms: Some(50.0),
            success: true,
            error: None,
        };

        stats.update(&success_result);
        assert_eq!(stats.consecutive_failures, 0);
        assert!(stats.health_score > 0.9);
    }

    #[test]
    fn test_health_score_degradation() {
        let mut stats = NodeHealthStats::new("node1".to_string());

        let failure_result = ProbeResult {
            node_id: "node1".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            protocol: ProbeProtocol::Tcp,
            rtt_ms: None,
            success: false,
            error: Some("Connection refused".to_string()),
        };

        stats.update(&failure_result);
        stats.update(&failure_result);
        stats.update(&failure_result);

        assert_eq!(stats.consecutive_failures, 3);
        assert!(!stats.is_healthy());
    }

    #[tokio::test]
    async fn test_tcp_probe_localhost() {
        let prober = ActiveProber::new(1000, 30);

        // This will fail unless there's a service on port 80
        let result = prober.probe_tcp("test", "127.0.0.1", 80, None).await;

        assert_eq!(result.protocol, ProbeProtocol::Tcp);
        assert_eq!(result.node_id, "test");
    }
}
