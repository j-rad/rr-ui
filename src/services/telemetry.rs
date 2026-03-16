// src/services/telemetry.rs
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use sysinfo::{CpuExt, DiskExt, NetworkExt, NetworksExt, System, SystemExt};
use tokio::sync::RwLock;

use crate::domain::models::TrafficHistoryPoint;
use crate::models::TrafficStats;

/// System telemetry data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemTelemetry {
    pub cpu_usage: f32,
    pub memory_total: u64,
    pub memory_used: u64,
    pub memory_percent: f32,
    pub disk_total: u64,
    pub disk_used: u64,
    pub disk_percent: f32,
    pub uptime: u64,
    pub load_average: (f64, f64, f64),
    pub net_up: u64,
    pub net_down: u64,
    pub net_sent: u64,
    pub net_recv: u64,
    pub tcp_count: u32,
    pub udp_count: u32,
}

use crate::rustray_client::RustRayClient;
use std::sync::{Mutex, OnceLock};

static GLOBAL_TELEMETRY: OnceLock<TelemetryService> = OnceLock::new();

/// Telemetry collector service
#[derive(Clone)]
pub struct TelemetryService {
    system: Arc<RwLock<System>>,
    last_telemetry: Arc<RwLock<Option<SystemTelemetry>>>,
    last_traffic: Arc<RwLock<Vec<TrafficStats>>>,
    traffic_history: Arc<RwLock<VecDeque<TrafficHistoryPoint>>>,
    rustray_client: Arc<Mutex<Option<RustRayClient>>>,
}

impl TelemetryService {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        Self {
            system: Arc::new(RwLock::new(system)),
            last_telemetry: Arc::new(RwLock::new(None)),
            last_traffic: Arc::new(RwLock::new(Vec::new())),
            traffic_history: Arc::new(RwLock::new(VecDeque::with_capacity(60))),
            rustray_client: Arc::new(Mutex::new(None)),
        }
    }

    pub fn global() -> &'static Self {
        GLOBAL_TELEMETRY.get_or_init(Self::new)
    }

    pub fn set_rustray_client(&self, client: RustRayClient) {
        let mut guard = self.rustray_client.lock().unwrap();
        *guard = Some(client);
    }

    /// Start background telemetry collection
    pub fn start(&self) {
        let system = self.system.clone();
        let last_telemetry = self.last_telemetry.clone();
        let last_traffic = self.last_traffic.clone();
        let traffic_history = self.traffic_history.clone();
        let rustray_client = self.rustray_client.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(1500));
            loop {
                interval.tick().await;

                // --- System Stats ---
                // Refresh system info
                {
                    let mut sys = system.write().await;
                    sys.refresh_cpu();
                    sys.refresh_memory();
                    sys.refresh_disks_list();
                    sys.refresh_disks();
                    sys.refresh_networks();
                }

                // Collect telemetry
                let telemetry = {
                    let sys = system.read().await;

                    // Using sysinfo 0.29 disk iteration
                    let mut disk_total = 0u64;
                    let mut disk_available = 0u64;

                    for disk in sys.disks() {
                        disk_total += disk.total_space();
                        disk_available += disk.available_space();
                    }

                    let disk_used = disk_total.saturating_sub(disk_available);

                    let disk_percent = if disk_total > 0 {
                        (disk_used as f32 / disk_total as f32) * 100.0
                    } else {
                        0.0
                    };

                    let memory_total = sys.total_memory();
                    let memory_used = sys.used_memory();
                    let memory_percent = if memory_total > 0 {
                        (memory_used as f32 / memory_total as f32) * 100.0
                    } else {
                        0.0
                    };

                    // Get CPU usage (average across all cores)
                    let cpu_usage = sys.global_cpu_info().cpu_usage();

                    // Get load average
                    let load_avg = sys.load_average();

                    SystemTelemetry {
                        cpu_usage,
                        memory_total,
                        memory_used,
                        memory_percent,
                        disk_total,
                        disk_used,
                        disk_percent,
                        uptime: sys.uptime(),
                        load_average: (load_avg.one, load_avg.five, load_avg.fifteen),
                        net_up: sys.networks().iter().map(|(_, n)| n.transmitted()).sum(),
                        net_down: sys.networks().iter().map(|(_, n)| n.received()).sum(),
                        net_sent: sys
                            .networks()
                            .iter()
                            .map(|(_, n)| n.total_transmitted())
                            .sum(),
                        net_recv: sys.networks().iter().map(|(_, n)| n.total_received()).sum(),
                        tcp_count: 0, // sysinfo doesn't utilize netstat, requires dedicated crate or procfs parsing
                        udp_count: 0,
                    }
                };

                // Store telemetry
                {
                    let mut last = last_telemetry.write().await;
                    *last = Some(telemetry.clone());

                    // Update History
                    let mut history = traffic_history.write().await;
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();

                    // sysinfo transmitted() is since last refresh
                    // Interval is 1.5s, so convert to per-second rate
                    let up_rate = (telemetry.net_up as f32 / 1.5) as u64;
                    let down_rate = (telemetry.net_down as f32 / 1.5) as u64;

                    history.push_back(TrafficHistoryPoint {
                        timestamp,
                        up_rate,
                        down_rate,
                    });

                    if history.len() > 60 {
                        history.pop_front();
                    }
                }

                // --- RustRay Stats ---
                let mut stats_vec = Vec::new();
                let mut client_opt = rustray_client.lock().unwrap().clone();

                if let Some(mut client) = client_opt {
                    match client.get_traffic_stats(false).await {
                        Ok(stats) => {
                            let mut total_up = 0i64;
                            let mut total_down = 0i64;

                            for stat in stats {
                                if stat.name.contains("uplink")
                                    || stat.name.ends_with(">>>traffic>>>uplink")
                                {
                                    total_up += stat.value;
                                } else if stat.name.contains("downlink")
                                    || stat.name.ends_with(">>>traffic>>>downlink")
                                {
                                    total_down += stat.value;
                                }
                            }

                            stats_vec.push(TrafficStats {
                                name: "uplink".to_string(),
                                value: total_up,
                            });
                            stats_vec.push(TrafficStats {
                                name: "downlink".to_string(),
                                value: total_down,
                            });
                        }
                        Err(_) => {
                            // Handle offline/error
                        }
                    }
                }

                {
                    let mut traffic = last_traffic.write().await;
                    if !stats_vec.is_empty() {
                        *traffic = stats_vec;
                    }
                }
            }
        });
    }

    /// Get latest telemetry data
    pub async fn get_telemetry(&self) -> Option<SystemTelemetry> {
        let telemetry = self.last_telemetry.read().await;
        telemetry.clone()
    }

    pub async fn get_traffic_stats(&self) -> Vec<TrafficStats> {
        let traffic = self.last_traffic.read().await;
        traffic.clone()
    }

    pub async fn get_traffic_history(&self) -> Vec<TrafficHistoryPoint> {
        let history = self.traffic_history.read().await;
        history
            .iter()
            .cloned()
            .collect::<Vec<TrafficHistoryPoint>>()
    }
}

impl Default for TelemetryService {
    fn default() -> Self {
        Self::new()
    }
}

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkStats {
    pub total_up: u64,
    pub total_down: u64,
    pub active_connections: usize,
}

/// Get network statistics from database
pub async fn get_network_stats(db: &crate::db::DbClient) -> Result<NetworkStats> {
    #[cfg(feature = "server")]
    {
        use crate::models::Inbound;

        let inbounds: Vec<Inbound> = db.client.select("inbound").await?;

        let (total_up, total_down) = inbounds.iter().fold((0u64, 0u64), |acc, inbound| {
            (
                acc.0 + inbound.up_bytes as u64,
                acc.1 + inbound.down_bytes as u64,
            )
        });

        // Count active clients
        let active_connections = inbounds
            .iter()
            .filter(|i| i.enable)
            .map(|i| {
                // Use the helper method for simpler client counting
                i.settings.clients().map(|c| c.len()).unwrap_or(0)
            })
            .sum();

        Ok(NetworkStats {
            total_up,
            total_down,
            active_connections,
        })
    }

    #[cfg(not(feature = "server"))]
    {
        Ok(NetworkStats {
            total_up: 0,
            total_down: 0,
            active_connections: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_telemetry_service() {
        let service = TelemetryService::new();
        service.start();

        // Wait for first collection
        tokio::time::sleep(Duration::from_secs(3)).await;

        let telemetry = service.get_telemetry().await;
        assert!(telemetry.is_some());

        let data = telemetry.unwrap();
        assert!(data.cpu_usage >= 0.0);
        assert!(data.memory_total > 0);
    }
}
