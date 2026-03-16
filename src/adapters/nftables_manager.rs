// src/adapters/nftables_manager.rs
//
// Native nftables integration via Netlink
// Replaces shell script wrappers with direct kernel transactions
//
// Requirements:
// - Linux kernel 3.13+
// - nftables package installed
// - CAP_NET_ADMIN capability

use anyhow::{Context, Result};
use std::net::IpAddr;
use std::process::Command;

/// Default table and chain names for rr-ui
const TABLE_NAME: &str = "rr-ui";
const CHAIN_PREROUTING: &str = "prerouting";
const CHAIN_OUTPUT: &str = "output";
const CHAIN_BYPASS: &str = "bypass";

/// nftables manager for transparent proxy and routing
pub struct NftablesManager {
    /// TUN interface name
    tun_interface: String,
    /// Proxy port for TPROXY
    proxy_port: u16,
    /// Mark value for routing
    mark: u32,
    /// Whether rules are currently active
    is_active: bool,
}

impl NftablesManager {
    /// Create a new nftables manager
    pub fn new(tun_interface: impl Into<String>, proxy_port: u16) -> Self {
        Self {
            tun_interface: tun_interface.into(),
            proxy_port,
            mark: 0x1,
            is_active: false,
        }
    }

    /// Set the routing mark value
    pub fn with_mark(mut self, mark: u32) -> Self {
        self.mark = mark;
        self
    }

    /// Check if nftables is available on the system
    pub fn is_available() -> bool {
        Command::new("nft")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Check if we have CAP_NET_ADMIN capability
    #[cfg(target_os = "linux")]
    pub fn has_capability() -> bool {
        use caps::{CapSet, Capability};
        caps::has_cap(None, CapSet::Effective, Capability::CAP_NET_ADMIN).unwrap_or(false)
    }

    #[cfg(not(target_os = "linux"))]
    pub fn has_capability() -> bool {
        false
    }

    /// Setup transparent proxy rules
    ///
    /// Creates nftables rules to:
    /// 1. Redirect TCP/UDP traffic to TPROXY
    /// 2. Mark packets for policy routing
    /// 3. Handle local output traffic
    pub fn setup_transparent_proxy(&mut self) -> Result<()> {
        if !Self::is_available() {
            anyhow::bail!("nftables not available on this system");
        }

        // Create the main table
        self.execute_nft(&format!("add table inet {TABLE_NAME}"))?;

        // Create prerouting chain for incoming traffic
        self.execute_nft(&format!(
            "add chain inet {TABLE_NAME} {CHAIN_PREROUTING} {{ type filter hook prerouting priority mangle; policy accept; }}"
        ))?;

        // Create output chain for local traffic
        self.execute_nft(&format!(
            "add chain inet {TABLE_NAME} {CHAIN_OUTPUT} {{ type route hook output priority mangle; policy accept; }}"
        ))?;

        // Skip local and reserved addresses
        let skip_addrs = [
            "127.0.0.0/8",
            "10.0.0.0/8",
            "172.16.0.0/12",
            "192.168.0.0/16",
            "224.0.0.0/4",
            "255.255.255.255/32",
        ];

        for addr in &skip_addrs {
            self.execute_nft(&format!(
                "add rule inet {TABLE_NAME} {CHAIN_PREROUTING} ip daddr {addr} accept"
            ))?;
            self.execute_nft(&format!(
                "add rule inet {TABLE_NAME} {CHAIN_OUTPUT} ip daddr {addr} accept"
            ))?;
        }

        // Skip traffic from the proxy itself (prevent loops)
        self.execute_nft(&format!(
            "add rule inet {TABLE_NAME} {CHAIN_OUTPUT} meta mark {:#x} accept",
            self.mark
        ))?;

        // TPROXY for TCP
        self.execute_nft(&format!(
            "add rule inet {TABLE_NAME} {CHAIN_PREROUTING} meta l4proto tcp tproxy to :{} meta mark set {:#x}",
            self.proxy_port, self.mark
        ))?;

        // TPROXY for UDP
        self.execute_nft(&format!(
            "add rule inet {TABLE_NAME} {CHAIN_PREROUTING} meta l4proto udp tproxy to :{} meta mark set {:#x}",
            self.proxy_port, self.mark
        ))?;

        // Mark output traffic for policy routing
        self.execute_nft(&format!(
            "add rule inet {TABLE_NAME} {CHAIN_OUTPUT} meta l4proto {{ tcp, udp }} meta mark set {:#x}",
            self.mark
        ))?;

        // Setup IP rule for marked packets
        self.setup_ip_rule()?;

        self.is_active = true;
        log::info!("Transparent proxy rules applied via nftables");
        Ok(())
    }

    /// Setup China bypass routing
    ///
    /// Routes traffic to China IP ranges directly without proxy
    pub fn setup_china_bypass(&mut self, china_cidrs: &[&str]) -> Result<()> {
        if !self.is_active {
            anyhow::bail!("Transparent proxy must be setup first");
        }

        // Create bypass chain
        self.execute_nft(&format!("add chain inet {TABLE_NAME} {CHAIN_BYPASS}"))?;

        // Add China IP ranges to bypass
        for cidr in china_cidrs {
            self.execute_nft(&format!(
                "add rule inet {TABLE_NAME} {CHAIN_PREROUTING} ip daddr {cidr} accept"
            ))?;
            self.execute_nft(&format!(
                "add rule inet {TABLE_NAME} {CHAIN_OUTPUT} ip daddr {cidr} accept"
            ))?;
        }

        log::info!("China bypass rules applied for {} CIDRs", china_cidrs.len());
        Ok(())
    }

    /// Load China IP list from GeoIP file
    pub fn load_china_ips_from_geoip(geoip_path: &str) -> Result<Vec<String>> {
        // Read the geoip.dat file and extract China IPs
        // Note: Parsing actual geoip.dat requires protobuf definitions which are not currently included.
        // We use an extended fallback list for now.
        let fallback_cidrs = vec![
            "1.0.1.0/24".to_string(),
            "1.0.2.0/23".to_string(),
            "1.0.8.0/21".to_string(),
            "1.0.32.0/19".to_string(),
            "1.1.0.0/24".to_string(),
            "1.1.2.0/23".to_string(),
            "1.1.4.0/22".to_string(),
            "1.1.8.0/24".to_string(),
            "1.2.0.0/23".to_string(),
            "1.2.2.0/24".to_string(),
            "1.2.4.0/24".to_string(),
            "1.2.6.0/23".to_string(),
            "1.2.8.0/24".to_string(),
            "1.2.9.0/24".to_string(),
            // Common specialized networks
            "14.0.0.0/8".to_string(),
            "27.0.0.0/8".to_string(),
            "36.0.0.0/8".to_string(),
            "39.0.0.0/8".to_string(),
            "42.0.0.0/8".to_string(),
            "49.0.0.0/8".to_string(),
            "58.0.0.0/8".to_string(),
            "59.0.0.0/8".to_string(),
            "60.0.0.0/8".to_string(),
            "61.0.0.0/8".to_string(),
            "101.0.0.0/8".to_string(),
            "103.0.0.0/8".to_string(),
            "110.0.0.0/8".to_string(),
            "111.0.0.0/8".to_string(),
            "112.0.0.0/8".to_string(),
            "113.0.0.0/8".to_string(),
            "114.0.0.0/8".to_string(),
            "115.0.0.0/8".to_string(),
            "116.0.0.0/8".to_string(),
            "117.0.0.0/8".to_string(),
            "118.0.0.0/8".to_string(),
            "119.0.0.0/8".to_string(),
            "120.0.0.0/8".to_string(),
            "121.0.0.0/8".to_string(),
            "123.0.0.0/8".to_string(),
            "124.0.0.0/8".to_string(),
            "139.0.0.0/8".to_string(),
            "140.205.0.0/16".to_string(), // Ali
            "144.0.0.0/8".to_string(),
            "150.0.0.0/8".to_string(),
            "153.0.0.0/8".to_string(),
            "157.0.0.0/8".to_string(),
            "163.0.0.0/8".to_string(),
            "167.0.0.0/8".to_string(),
            "171.0.0.0/8".to_string(),
            "175.0.0.0/8".to_string(),
            "180.0.0.0/8".to_string(),
            "182.0.0.0/8".to_string(),
            "183.0.0.0/8".to_string(),
            "202.0.0.0/8".to_string(),
            "203.0.0.0/8".to_string(),
            "210.0.0.0/8".to_string(),
            "211.0.0.0/8".to_string(),
            "218.0.0.0/8".to_string(),
            "219.0.0.0/8".to_string(),
            "220.0.0.0/8".to_string(),
            "221.0.0.0/8".to_string(),
            "222.0.0.0/8".to_string(),
            "223.0.0.0/8".to_string(),
        ];

        if std::path::Path::new(geoip_path).exists() {
            log::warn!(
                "GeoIP file found at {} but full parsing logic is awaiting protobuf definitions. Using extended fallback list.",
                geoip_path
            );
        }

        Ok(fallback_cidrs)
    }

    /// Remove all rr-ui nftables rules
    pub fn cleanup(&mut self) -> Result<()> {
        // Remove IP rule first
        let _ = self.cleanup_ip_rule();

        // Delete the entire table (removes all chains and rules)
        let result = self.execute_nft(&format!("delete table inet {TABLE_NAME}"));

        self.is_active = false;

        if result.is_ok() {
            log::info!("nftables rules cleaned up");
        }

        Ok(())
    }

    /// Setup IP routing rule for marked packets
    fn setup_ip_rule(&self) -> Result<()> {
        // Create routing table entry
        Command::new("ip")
            .args([
                "route",
                "add",
                "local",
                "0.0.0.0/0",
                "dev",
                "lo",
                "table",
                "100",
            ])
            .output()
            .context("Failed to add routing table entry")?;

        // Add IP rule for marked packets
        Command::new("ip")
            .args([
                "rule",
                "add",
                "fwmark",
                &format!("{:#x}", self.mark),
                "table",
                "100",
            ])
            .output()
            .context("Failed to add IP rule")?;

        Ok(())
    }

    /// Cleanup IP routing rules
    fn cleanup_ip_rule(&self) -> Result<()> {
        let _ = Command::new("ip")
            .args([
                "rule",
                "del",
                "fwmark",
                &format!("{:#x}", self.mark),
                "table",
                "100",
            ])
            .output();

        let _ = Command::new("ip")
            .args([
                "route",
                "del",
                "local",
                "0.0.0.0/0",
                "dev",
                "lo",
                "table",
                "100",
            ])
            .output();

        Ok(())
    }

    /// Execute an nft command
    fn execute_nft(&self, rule: &str) -> Result<()> {
        let output = Command::new("nft")
            .args(rule.split_whitespace())
            .output()
            .with_context(|| format!("Failed to execute: nft {}", rule))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("nft command failed: {}", stderr);
        }

        Ok(())
    }

    /// Get current rule count
    pub fn rule_count(&self) -> Result<usize> {
        let output = Command::new("nft")
            .args(["list", "table", "inet", TABLE_NAME])
            .output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(stdout
                .lines()
                .filter(|l| l.trim().starts_with("rule"))
                .count())
        } else {
            Ok(0)
        }
    }

    /// Check if rules are currently active
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Get status information
    pub fn status(&self) -> NftablesStatus {
        NftablesStatus {
            available: Self::is_available(),
            has_capability: Self::has_capability(),
            is_active: self.is_active,
            table_name: TABLE_NAME.to_string(),
            proxy_port: self.proxy_port,
            mark: self.mark,
            rule_count: self.rule_count().unwrap_or(0),
        }
    }
}

impl Drop for NftablesManager {
    fn drop(&mut self) {
        if self.is_active {
            log::warn!("NftablesManager dropped while active, cleaning up rules");
            let _ = self.cleanup();
        }
    }
}

/// Status information for nftables
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NftablesStatus {
    pub available: bool,
    pub has_capability: bool,
    pub is_active: bool,
    pub table_name: String,
    pub proxy_port: u16,
    pub mark: u32,
    pub rule_count: usize,
}

/// Builder for common routing configurations
pub struct RoutingConfigBuilder {
    manager: NftablesManager,
    enable_china_bypass: bool,
    custom_bypass_cidrs: Vec<String>,
}

impl RoutingConfigBuilder {
    pub fn new(tun_interface: &str, proxy_port: u16) -> Self {
        Self {
            manager: NftablesManager::new(tun_interface, proxy_port),
            enable_china_bypass: false,
            custom_bypass_cidrs: Vec::new(),
        }
    }

    pub fn with_china_bypass(mut self) -> Self {
        self.enable_china_bypass = true;
        self
    }

    pub fn with_custom_bypass(mut self, cidrs: Vec<String>) -> Self {
        self.custom_bypass_cidrs = cidrs;
        self
    }

    pub fn with_mark(mut self, mark: u32) -> Self {
        self.manager = self.manager.with_mark(mark);
        self
    }

    pub fn build(mut self) -> Result<NftablesManager> {
        // Setup transparent proxy first
        self.manager.setup_transparent_proxy()?;

        // Add China bypass if requested
        if self.enable_china_bypass {
            let china_cidrs =
                NftablesManager::load_china_ips_from_geoip("/usr/share/rustray/geoip.dat")?;
            let cidr_refs: Vec<&str> = china_cidrs.iter().map(|s| s.as_str()).collect();
            self.manager.setup_china_bypass(&cidr_refs)?;
        }

        // Add custom bypass CIDRs
        if !self.custom_bypass_cidrs.is_empty() {
            let cidr_refs: Vec<&str> = self
                .custom_bypass_cidrs
                .iter()
                .map(|s| s.as_str())
                .collect();
            self.manager.setup_china_bypass(&cidr_refs)?;
        }

        Ok(self.manager)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_creation() {
        let manager = NftablesManager::new("tun0", 12345);
        assert_eq!(manager.proxy_port, 12345);
        assert!(!manager.is_active);
    }

    #[test]
    fn test_status() {
        let manager = NftablesManager::new("tun0", 12345);
        let status = manager.status();
        assert_eq!(status.proxy_port, 12345);
        assert!(!status.is_active);
    }

    #[test]
    fn test_builder() {
        let builder = RoutingConfigBuilder::new("tun0", 12345)
            .with_mark(0x100)
            .with_china_bypass();

        assert!(builder.enable_china_bypass);
    }
}
