use anyhow::{Context, Result};
use log::info;
use std::process::Command;

pub struct IptablesManager;

impl IptablesManager {
    /// Set up TPROXY rules for transparent proxying
    ///
    /// # Arguments
    /// * `tproxy_port` - The port RustRay Core is listening on for TPROXY/Dokodemo-door
    /// * `mark` - FWMark to use (e.g. 1)
    pub fn setup_tproxy(tproxy_port: u16, mark: u32) -> Result<()> {
        info!(
            "Setting up TPROXY rules on port {} with mark {}",
            tproxy_port, mark
        );

        // 1. Create custom chain
        Self::run_iptables(&["-t", "mangle", "-N", "RUSTRAY"])?;

        // 2. Ignore RustRay traffic (avoid loops) - assuming RustRay runs as specific user or similar
        // Ideally we mark output traffic from RustRay process?
        // For now, we assume simple TPROXY setup for INCOMING/PREROUTING mostly,
        // or OUTPUT for local traffic (requires creating route for looopback).

        // Return local and loopback
        Self::run_iptables(&[
            "-t",
            "mangle",
            "-A",
            "RUSTRAY",
            "-d",
            "127.0.0.1/32",
            "-j",
            "RETURN",
        ])?;
        Self::run_iptables(&[
            "-t",
            "mangle",
            "-A",
            "RUSTRAY",
            "-d",
            "224.0.0.0/4",
            "-j",
            "RETURN",
        ])?;
        Self::run_iptables(&[
            "-t",
            "mangle",
            "-A",
            "RUSTRAY",
            "-d",
            "255.255.255.255/32",
            "-j",
            "RETURN",
        ])?;

        // Mark remaining traffic
        Self::run_iptables(&[
            "-t",
            "mangle",
            "-A",
            "RUSTRAY",
            "-p",
            "tcp",
            "-j",
            "TPROXY",
            "--on-port",
            &tproxy_port.to_string(),
            "--tproxy-mark",
            &mark.to_string(),
        ])?;

        Self::run_iptables(&[
            "-t",
            "mangle",
            "-A",
            "RUSTRAY",
            "-p",
            "udp",
            "-j",
            "TPROXY",
            "--on-port",
            &tproxy_port.to_string(),
            "--tproxy-mark",
            &mark.to_string(),
        ])?;

        // Apply chain to PREROUTING
        Self::run_iptables(&["-t", "mangle", "-A", "PREROUTING", "-j", "RUSTRAY"])?;

        // For LOCAL traffic (OUTPUT), it's more complex (requires rerouting to loopback).
        // Self::setup_local_tproxy(mark)?;

        Ok(())
    }

    /// Clean up TPROXY rules
    pub fn clear_tproxy() -> Result<()> {
        info!("Clearing TPROXY rules");
        // Flush and delete chain
        // Ignore errors if chain doesn't exist
        let _ = Self::run_iptables(&["-t", "mangle", "-D", "PREROUTING", "-j", "RUSTRAY"]);
        let _ = Self::run_iptables(&["-t", "mangle", "-F", "RUSTRAY"]);
        let _ = Self::run_iptables(&["-t", "mangle", "-X", "RUSTRAY"]);
        Ok(())
    }

    fn run_iptables(args: &[&str]) -> Result<()> {
        let output = Command::new("iptables")
            .args(args)
            .output()
            .with_context(|| format!("Failed to run iptables with args: {:?}", args))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Don't error if it's just "Chain already exists" etc, but simple check is hard.
            // For now, log warning or error.
            if stderr.contains("Chain already exists") {
                return Ok(());
            }
            anyhow::bail!("iptables error: {}", stderr);
        }
        Ok(())
    }
}
