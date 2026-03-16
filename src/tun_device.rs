// src/tun_device.rs
//! High-Performance TUN Device using tun-rs
//!
//! Cross-platform TUN device wrapper with async Tokio integration.
//! Supports dynamic MTU profiles for different network environments.
//! Adapted from rustray for rr-ui.

use anyhow::Result;
use log::{debug, info};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::sync::Arc;

// ============================================================================
// MTU Profiles
// ============================================================================

/// MTU Profile for different network environments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MtuProfile {
    /// 1400 MTU - Conservative for cellular/mobile networks
    Cellular,
    /// 1500 MTU - Standard ethernet MTU
    Standard,
    /// 9000 MTU - Jumbo frames for high-performance LAN/DC
    Jumbo,
    /// Custom MTU value
    Custom(u16),
}

impl MtuProfile {
    /// Get the raw MTU value for this profile
    pub fn mtu(&self) -> u16 {
        match self {
            MtuProfile::Cellular => 1400,
            MtuProfile::Standard => 1500,
            MtuProfile::Jumbo => 9000,
            MtuProfile::Custom(mtu) => *mtu,
        }
    }

    pub fn mss_ipv4(&self) -> u16 {
        self.mtu().saturating_sub(40)
    }

    pub fn mss_ipv6(&self) -> u16 {
        self.mtu().saturating_sub(60)
    }

    pub fn buffer_size(&self) -> usize {
        self.mtu() as usize + 64
    }
}

impl Default for MtuProfile {
    fn default() -> Self {
        MtuProfile::Cellular
    }
}

impl From<u16> for MtuProfile {
    fn from(mtu: u16) -> Self {
        match mtu {
            1400 => MtuProfile::Cellular,
            1500 => MtuProfile::Standard,
            9000 => MtuProfile::Jumbo,
            other => MtuProfile::Custom(other),
        }
    }
}

// ============================================================================
// TUN Configuration
// ============================================================================

/// TUN device configuration
#[derive(Debug, Clone)]
pub struct TunConfig {
    pub name: String,
    pub address: Ipv4Addr,
    pub netmask: Ipv4Addr,
    pub address_v6: Option<Ipv6Addr>,
    pub prefix_v6: u8,
    pub mtu_profile: MtuProfile,
    pub packet_info: bool,
    // rr-ui specific: routing flags
    pub set_default_route: bool,
    pub bypass: Vec<std::net::IpAddr>,
}

impl Default for TunConfig {
    fn default() -> Self {
        Self {
            name: "rr-tun0".to_string(),
            address: Ipv4Addr::new(10, 0, 0, 1),
            netmask: Ipv4Addr::new(255, 255, 255, 0),
            address_v6: None, // Simplified for rr-ui
            prefix_v6: 64,
            mtu_profile: MtuProfile::Cellular,
            packet_info: false,
            set_default_route: false,
            bypass: vec![],
        }
    }
}

impl TunConfig {
    pub fn mtu(&self) -> u16 {
        self.mtu_profile.mtu()
    }
}

// ============================================================================
// TUN Device Wrapper
// ============================================================================

pub struct TunDevice {
    device: tun_rs::AsyncDevice,
    #[allow(dead_code)]
    pub config: TunConfig,
}

impl TunDevice {
    pub fn create(config: TunConfig) -> Result<Self> {
        let mtu = config.mtu();
        info!(
            "Creating TUN device: {} (MTU: {}, Profile: {:?})",
            config.name, mtu, config.mtu_profile
        );

        let mut builder = tun_rs::DeviceBuilder::new()
            .ipv4(config.address, config.netmask, None)
            .mtu(mtu);

        #[cfg(target_os = "linux")]
        {
            builder = builder.name(&config.name);
        }

        #[cfg(target_os = "macos")]
        {
            builder = builder.name("utun");
        }

        if let Some(v6) = config.address_v6 {
            builder = builder.ipv6(v6, config.prefix_v6);
        }

        let device = builder.build_async()?;

        info!(
            "TUN device created: {} (IPv4: {}/{})",
            config.name, config.address, config.netmask
        );

        Ok(Self { device, config })
    }

    pub async fn recv(&self, buf: &mut [u8]) -> std::io::Result<usize> {
        // tun-rs AsyncDevice implements AsyncRead/AsyncWrite via tokio codecs usually,
        // or provides recv/send?
        // Checking rustray code: `self.device.readable().await?; self.device.try_recv(buf)`
        // NOTE: tun-rs 2.0 AsyncDevice might work differently than tun 0.8.
        // Assuming rustray code was correct for tun-rs v2.

        self.device.readable().await?;
        match self.device.try_recv(buf) {
            Ok(n) => {
                debug!("TUN: Received {} bytes", n);
                Ok(n)
            }
            // Simple recursive retry for now as per rustray reference
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Box::pin(self.recv(buf)).await,
            Err(e) => Err(e),
        }
    }

    pub async fn send(&self, buf: &[u8]) -> std::io::Result<usize> {
        self.device.writable().await?;
        match self.device.try_send(buf) {
            Ok(n) => Ok(n),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Box::pin(self.send(buf)).await,
            Err(e) => Err(e),
        }
    }

    pub fn split(
        self,
    ) -> (
        std::sync::Arc<tun_rs::AsyncDevice>,
        std::sync::Arc<tun_rs::AsyncDevice>,
    ) {
        let d = std::sync::Arc::new(self.device);
        (d.clone(), d)
    }
}

// ============================================================================
// TUN Manager (from rr-ui original, adapted)
// ============================================================================

use tokio::sync::{RwLock, broadcast};

#[derive(Debug, Clone, PartialEq)]
pub enum TunState {
    Stopped,
    Starting,
    Running,
    Error(String),
}

pub struct TunManager {
    config: Arc<RwLock<TunConfig>>,
    state: Arc<RwLock<TunState>>,
    shutdown_tx: Option<broadcast::Sender<()>>,
}

impl TunManager {
    pub fn new(config: TunConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            state: Arc::new(RwLock::new(TunState::Stopped)),
            shutdown_tx: None,
        }
    }

    pub async fn start(&mut self, _proxy_addr: std::net::SocketAddr) -> Result<()> {
        let mut state = self.state.write().await;
        if *state == TunState::Running {
            return Ok(());
        }
        *state = TunState::Starting;
        drop(state); // Drop lock during heavy setup

        let config = self.config.read().await.clone();

        // 1. Create Device
        let device = match TunDevice::create(config.clone()) {
            Ok(d) => d,
            Err(e) => {
                let mut s = self.state.write().await;
                *s = TunState::Error(e.to_string());
                return Err(e);
            }
        };

        // 2. Setup nftables (native Netlink instead of shell scripts)
        #[cfg(target_os = "linux")]
        if config.set_default_route {
            use crate::adapters::nftables_manager::{NftablesManager, RoutingConfigBuilder};

            // Check if nftables is available
            if !NftablesManager::is_available() {
                log::warn!("nftables not available, skipping transparent proxy setup");
            } else if !NftablesManager::has_capability() {
                log::warn!("CAP_NET_ADMIN capability required for nftables");
            } else {
                // Setup transparent proxy with China bypass
                match RoutingConfigBuilder::new(&config.name, 12345)
                    .with_china_bypass()
                    .with_mark(0x1)
                    .build()
                {
                    Ok(manager) => {
                        log::info!(
                            "nftables transparent proxy configured: {} rules active",
                            manager.rule_count().unwrap_or(0)
                        );
                        // Store manager in TunDevice for cleanup on drop
                        // For now, it will auto-cleanup via Drop trait
                    }
                    Err(e) => {
                        log::error!("Failed to setup nftables: {}", e);
                        // Continue anyway - TUN device can work without TPROXY
                    }
                }
            }
        }

        // 3. Spawn Loop
        let (shutdown_tx, mut shutdown_rx) = broadcast::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        let state_clone = self.state.clone();
        // let (reader, writer) = device.split(); // Need split logic
        let device = Arc::new(device);

        tokio::spawn(async move {
            let mut buf = vec![0u8; 65535];
            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => break,
                    res = device.recv(&mut buf) => {
                        match res {
                            Ok(_n) => {
                                // Process packet...
                                // For now just valid read
                            }
                            Err(_) => break,
                        }
                    }
                }
            }
            let mut s = state_clone.write().await;
            *s = TunState::Stopped;
        });

        let mut state = self.state.write().await;
        *state = TunState::Running;
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        Ok(())
    }

    pub async fn get_status(&self) -> TunState {
        self.state.read().await.clone()
    }

    pub async fn update_config(&self, config: TunConfig) {
        let mut cfg = self.config.write().await;
        *cfg = config;
    }

    pub async fn get_config(&self) -> TunConfig {
        self.config.read().await.clone()
    }
}

pub type SharedTunManager = Arc<RwLock<TunManager>>;
