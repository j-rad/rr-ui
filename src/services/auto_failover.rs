// src/services/auto_failover.rs
//! Predictive Failover Engine
//!
//! Continuously probes node availability from Iranian vantage-point IP ranges
//! and triggers automated DNS record swaps (Cloudflare or generic provider)
//! when a node is detected as null-routed.
//!
//! # Architecture
//!
//! ```text
//!  ┌──────────────────────────────────────────────────────────┐
//!  │  PredictiveFailoverEngine                                │
//!  │                                                          │
//!  │  spawn_probe_loop()  ──► ActiveProber (per node)         │
//!  │       │                       │                          │
//!  │       ▼                       ▼                          │
//!  │  AutoFailover SM  ◄──  ProbeResult                       │
//!  │       │                                                  │
//!  │       └──► DnsSwapClient.swap_record()                   │
//!  │                 (Cloudflare / Generic provider API)      │
//!  └──────────────────────────────────────────────────────────┘
//! ```

use crate::services::active_probing::{ActiveProber, NodeHealthStats, ProbeProtocol, ProbeResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tokio::time::Duration;

// ──────────────────────────────── Node state machine ─────────────────────────

/// Per-node state within the failover system.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NodeState {
    Active,
    Degraded,
    Failed,
    Recovering,
    Standby,
}

/// A record of a failover transition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailoverEvent {
    pub event_id: String,
    pub timestamp: i64,
    pub event_type: FailoverEventType,
    pub from_node: String,
    pub to_node: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailoverEventType {
    NodeFailed,
    NodeRecovered,
    TrafficSwitched,
    RollbackInitiated,
}

/// Tunable thresholds for the state machine.
#[derive(Debug, Clone)]
pub struct FailoverConfig {
    /// How many consecutive failed probes before marking a node Failed.
    pub failure_threshold: u32,
    /// How many consecutive successful probes before Recovering → Active.
    pub recovery_threshold: u32,
    /// Minimum seconds between two failovers for the same node (cooldown).
    pub cooldown_seconds: u64,
    /// How often to run the probing loop.
    pub probe_interval_secs: u64,
    /// Timeout per individual TCP probe (ms).
    pub probe_timeout_ms: u64,
    /// Packet loss threshold (0.0 - 1.0)
    pub packet_loss_threshold: f32,
}

impl Default for FailoverConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 3,
            recovery_threshold: 5,
            cooldown_seconds: 300,
            probe_interval_secs: 30,
            probe_timeout_ms: 3000,
            packet_loss_threshold: 0.20, // 20% packet loss triggers failover
        }
    }
}

// ─────────────────────────── Pure state machine ───────────────────────────────

/// Core failover automaton — pure, no I/O.
pub struct AutoFailover {
    config: FailoverConfig,
    node_states: HashMap<String, NodeState>,
    failure_counts: HashMap<String, u32>,
    recovery_counts: HashMap<String, u32>,
    last_failover: HashMap<String, i64>,
    active_node: Option<String>,
    standby_nodes: Vec<String>,
}

impl AutoFailover {
    pub fn new(config: FailoverConfig) -> Self {
        Self {
            config,
            node_states: HashMap::new(),
            failure_counts: HashMap::new(),
            recovery_counts: HashMap::new(),
            last_failover: HashMap::new(),
            active_node: None,
            standby_nodes: Vec::new(),
        }
    }

    /// Register a managed node.
    pub fn register_node(&mut self, node_id: String, is_primary: bool) {
        if is_primary {
            self.active_node = Some(node_id.clone());
            self.node_states.insert(node_id.clone(), NodeState::Active);
        } else {
            self.standby_nodes.push(node_id.clone());
            self.node_states.insert(node_id.clone(), NodeState::Standby);
        }
        self.failure_counts.insert(node_id.clone(), 0);
        self.recovery_counts.insert(node_id, 0);
    }

    /// Feed one probe result into the state machine resulting in zero or more events.
    pub fn process_probe_result(
        &mut self,
        node_id: &str,
        success: bool,
        packet_loss: Option<f32>,
    ) -> Vec<FailoverEvent> {
        // Check packet loss threshold if provided
        let is_lossy = packet_loss
            .map(|loss| loss > self.config.packet_loss_threshold)
            .unwrap_or(false);

        if success && !is_lossy {
            self.handle_success(node_id)
        } else {
            self.handle_failure(node_id, is_lossy)
        }
    }

    fn handle_success(&mut self, node_id: &str) -> Vec<FailoverEvent> {
        let mut events = Vec::new();

        self.failure_counts.insert(node_id.to_string(), 0);
        let recovery_count = self.recovery_counts.entry(node_id.to_string()).or_insert(0);
        *recovery_count += 1;

        let current_state = self.node_states.get(node_id).cloned();

        match current_state {
            Some(NodeState::Failed) | Some(NodeState::Degraded) => {
                if *recovery_count >= self.config.recovery_threshold {
                    self.node_states
                        .insert(node_id.to_string(), NodeState::Recovering);

                    events.push(FailoverEvent {
                        event_id: format!("event_{}_{}", node_id, chrono::Utc::now().timestamp()),
                        timestamp: chrono::Utc::now().timestamp(),
                        event_type: FailoverEventType::NodeRecovered,
                        from_node: node_id.to_string(),
                        to_node: None,
                        reason: format!("{} consecutive successful probes", recovery_count),
                    });

                    self.recovery_counts.insert(node_id.to_string(), 0);
                }
            }
            Some(NodeState::Recovering) => {
                if self.should_rollback(node_id) {
                    events.extend(self.initiate_rollback(node_id));
                }
            }
            _ => {}
        }

        events
    }

    fn handle_failure(&mut self, node_id: &str, is_lossy: bool) -> Vec<FailoverEvent> {
        let mut events = Vec::new();

        self.recovery_counts.insert(node_id.to_string(), 0);
        let failure_count = self.failure_counts.entry(node_id.to_string()).or_insert(0);
        *failure_count += 1;

        let current_state = self.node_states.get(node_id).cloned();
        let fc = *failure_count;

        match current_state {
            Some(NodeState::Active) | Some(NodeState::Degraded)
                if is_lossy || fc >= self.config.failure_threshold =>
            {
                self.node_states
                    .insert(node_id.to_string(), NodeState::Failed);

                let reason = if is_lossy {
                    format!(
                        "Packet loss exceeded {:.0}%",
                        self.config.packet_loss_threshold * 100.0
                    )
                } else {
                    format!("{} consecutive probe failures", fc)
                };

                events.push(FailoverEvent {
                    event_id: format!("event_{}_{}", node_id, chrono::Utc::now().timestamp()),
                    timestamp: chrono::Utc::now().timestamp(),
                    event_type: FailoverEventType::NodeFailed,
                    from_node: node_id.to_string(),
                    to_node: None,
                    reason: reason.clone(),
                });

                if let Some(standby) = self.get_best_standby() {
                    events.extend(self.switch_traffic(node_id, &standby.clone(), reason));
                }
            }
            Some(NodeState::Active) => {
                // Below threshold; degrade only
                self.node_states
                    .insert(node_id.to_string(), NodeState::Degraded);
            }
            _ => {}
        }

        events
    }

    fn switch_traffic(&mut self, from: &str, to: &str, reason: String) -> Vec<FailoverEvent> {
        let now = chrono::Utc::now().timestamp();

        if let Some(last) = self.last_failover.get(from) {
            if now - last < self.config.cooldown_seconds as i64 {
                return Vec::new();
            }
        }

        self.node_states.insert(from.to_string(), NodeState::Failed);
        self.node_states.insert(to.to_string(), NodeState::Active);
        self.active_node = Some(to.to_string());
        self.last_failover.insert(from.to_string(), now);

        vec![FailoverEvent {
            event_id: format!("event_failover_{}", now),
            timestamp: now,
            event_type: FailoverEventType::TrafficSwitched,
            from_node: from.to_string(),
            to_node: Some(to.to_string()),
            reason,
        }]
    }

    fn should_rollback(&self, node_id: &str) -> bool {
        self.last_failover
            .get(node_id)
            .map(|last| {
                chrono::Utc::now().timestamp() - last >= self.config.cooldown_seconds as i64
            })
            .unwrap_or(false)
    }

    fn initiate_rollback(&mut self, node_id: &str) -> Vec<FailoverEvent> {
        let Some(current_active) = self.active_node.clone() else {
            return Vec::new();
        };
        if current_active == node_id {
            return Vec::new();
        }

        self.node_states
            .insert(node_id.to_string(), NodeState::Active);
        self.node_states
            .insert(current_active.clone(), NodeState::Standby);
        self.active_node = Some(node_id.to_string());

        vec![FailoverEvent {
            event_id: format!("event_rollback_{}", chrono::Utc::now().timestamp()),
            timestamp: chrono::Utc::now().timestamp(),
            event_type: FailoverEventType::RollbackInitiated,
            from_node: current_active,
            to_node: Some(node_id.to_string()),
            reason: "Original node recovered".to_string(),
        }]
    }

    fn get_best_standby(&self) -> Option<String> {
        self.standby_nodes
            .iter()
            .find(|id| {
                matches!(
                    self.node_states.get(*id),
                    Some(NodeState::Standby) | Some(NodeState::Recovering)
                )
            })
            .cloned()
    }

    pub fn get_active_node(&self) -> Option<&String> {
        self.active_node.as_ref()
    }

    pub fn get_node_state(&self, node_id: &str) -> Option<&NodeState> {
        self.node_states.get(node_id)
    }
}

// ──────────────────────────── Cloudflare DNS swap ────────────────────────────

/// Cloudflare API credentials and target zone.
#[derive(Debug, Clone)]
pub struct CloudflareConfig {
    /// Bearer token (CF_API_TOKEN).
    pub api_token: String,
    /// Zone ID of the domain being managed.
    pub zone_id: String,
    /// DNS record name to swap (e.g. "vpn.example.com").
    pub record_name: String,
}

/// Typed Cloudflare DNS record — only `A` records are relevant.
#[derive(Debug, Serialize, Deserialize)]
struct CloudflareRecord {
    id: String,
    #[serde(rename = "type")]
    record_type: String,
    name: String,
    content: String,
    proxied: bool,
    ttl: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct CloudflareListResponse {
    result: Vec<CloudflareRecord>,
    success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct CfPatchBody {
    content: String,
    ttl: u32,
}

/// Stateless Cloudflare DNS swap client.
pub struct CloudflareDnsSwap {
    config: CloudflareConfig,
    client: reqwest::Client,
}

impl CloudflareDnsSwap {
    pub fn new(config: CloudflareConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// Locate the A-record for `record_name` and set its content to `new_ip`.
    ///
    /// Uses the Cloudflare v4 API:
    ///   `PATCH /zones/{zone}/dns_records/{record_id}`
    pub async fn swap_to_ip(&self, new_ip: &str) -> anyhow::Result<()> {
        // 1. List DNS records to find the record ID.
        let list_url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records?type=A&name={}",
            self.config.zone_id, self.config.record_name
        );

        let list_resp: CloudflareListResponse = self
            .client
            .get(&list_url)
            .bearer_auth(&self.config.api_token)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let record =
            list_resp.result.into_iter().next().ok_or_else(|| {
                anyhow::anyhow!("DNS record '{}' not found", self.config.record_name)
            })?;

        if record.content == new_ip {
            // Already pointing at the right IP — no-op.
            return Ok(());
        }

        // 2. Patch the record.
        let patch_url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
            self.config.zone_id, record.id
        );

        self.client
            .patch(&patch_url)
            .bearer_auth(&self.config.api_token)
            .json(&CfPatchBody {
                content: new_ip.to_string(),
                ttl: 60, // 1 minute — fast propagation
            })
            .send()
            .await?
            .error_for_status()?;

        log::info!(
            "[failover] DNS record {} swapped {} → {}",
            self.config.record_name,
            record.content,
            new_ip
        );

        Ok(())
    }
}

// ─────────────────────── Node descriptor for the engine ──────────────────────

/// A managed node with its connectivity endpoints and standby IP.
#[derive(Debug, Clone)]
pub struct ManagedNode {
    pub node_id: String,
    /// Public IP / hostname the prober connects to.
    pub probe_host: String,
    /// Port to probe (TCP).
    pub probe_port: u16,
    /// Standby IP to swap the DNS record to if this node fails.
    pub standby_ip: String,
    /// Whether this is the primary (active) node at startup.
    pub is_primary: bool,
}

// ────────────────────── Iranian vantage-point source IPs ─────────────────────

/// A representative sample of Iranian IP prefixes used as bind / SOCKS source
/// for probes.  The engine uses these to simulate in-country reachability.
///
/// Format: (prefix_cidr, description)
pub const IRANIAN_PROBE_PREFIXES: &[(&str, &str)] = &[
    ("5.34.0.0/16", "Irancell / MTN Irancell"),
    ("5.200.0.0/16", "Pars Online"),
    ("80.191.0.0/16", "Shatel"),
    ("91.98.0.0/16", "Rightel"),
    ("91.108.0.0/16", "Hamrah Aval / MCI"),
    ("94.74.128.0/18", "Afranet"),
    ("185.55.224.0/22", "Respina"),
    ("188.121.96.0/19", "Asiatech"),
    ("213.108.96.0/19", "TCI / Telecommunication Co."),
];

/// Pick a concrete probe source IP within a representative Iranian range.
///
/// The returned IP is not bound or routed in any way — it is bundled with the
/// returned `ProbeConfig` so that a SOCKS relay or tun device sending from that
/// source can simulate in-country visibility.
pub fn pick_iranian_source_ip(prefix: &str) -> String {
    // Parse the first usable host from the CIDR for simplicity.
    // In production this would randomise within the prefix.
    let base = prefix.split('/').next().unwrap_or("5.34.0.1");
    let octets: Vec<&str> = base.split('.').collect();
    if octets.len() == 4 {
        // Use .1 host of the /16 or smaller prefix
        format!("{}.{}.1.1", octets[0], octets[1])
    } else {
        base.to_string()
    }
}

// ──────────────────────── Orchestrating engine ───────────────────────────────

/// `PredictiveFailoverEngine` wires together:
///  - An `ActiveProber` that fires TCP probes every N seconds
///  - An `AutoFailover` state machine that tracks per-node health
///  - A `CloudflareDnsSwap` client that pushes record updates when a failover fires
pub struct PredictiveFailoverEngine {
    state_machine: Arc<Mutex<AutoFailover>>,
    prober: Arc<ActiveProber>,
    dns_swap: Option<Arc<CloudflareDnsSwap>>,
    nodes: Vec<ManagedNode>,
    /// Channel used to receive probe results from spawned tasks.
    result_tx: mpsc::UnboundedSender<ProbeResult>,
    result_rx: Option<mpsc::UnboundedReceiver<ProbeResult>>,
    health_stats: Arc<Mutex<HashMap<String, NodeHealthStats>>>,
}

impl PredictiveFailoverEngine {
    /// Create the engine with an optional Cloudflare swap backend.
    pub fn new(config: FailoverConfig, cf_config: Option<CloudflareConfig>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let sm = AutoFailover::new(config.clone());

        Self {
            state_machine: Arc::new(Mutex::new(sm)),
            prober: Arc::new(ActiveProber::new(
                config.probe_timeout_ms,
                config.probe_interval_secs,
            )),
            dns_swap: cf_config.map(|c| Arc::new(CloudflareDnsSwap::new(c))),
            nodes: Vec::new(),
            result_tx: tx,
            result_rx: Some(rx),
            health_stats: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a node and start its probe loop.
    pub fn add_node(&mut self, node: ManagedNode) {
        {
            // Register in the state machine synchronously (we hold a blocking ref here
            // — it is safe because the event loop hasn't started yet).
            let sm_ref = self.state_machine.clone();
            let node_id = node.node_id.clone();
            let is_primary = node.is_primary;
            tokio::task::block_in_place(|| {
                let rt = tokio::runtime::Handle::current();
                rt.block_on(async move {
                    let mut sm = sm_ref.lock().await;
                    sm.register_node(node_id, is_primary);
                })
            });
        }

        self.nodes.push(node);
    }

    /// Start the engine.  This consumes the `result_rx` half of the channel
    /// and spawns:
    ///   1. One probe task per node (fires TCP probes and routes results onto `result_tx`).
    ///   2. One dispatch task that reads results, drives the state machine, and
    ///      triggers DNS swaps when failover events occur.
    ///
    /// Call this once, then the engine runs until the `JoinHandle` is awaited / aborted.
    pub async fn run(mut self) -> Vec<tokio::task::JoinHandle<()>> {
        let mut handles = Vec::new();

        // ── Spawn one probe loop per node ──────────────────────────────────────
        for node in &self.nodes {
            let tx = self.result_tx.clone();
            let prober = self.prober.clone();
            let node_id = node.node_id.clone();
            let host = node.probe_host.clone();
            let port = node.probe_port;

            // Pick a random Iranian source IP for this node's probes
            // In a real deployment, this would cycle or be configurable per node
            let (prefix, _) = IRANIAN_PROBE_PREFIXES[0]; // Just pick first for now
            let source_ip = Some(pick_iranian_source_ip(prefix));

            let handle = tokio::spawn(async move {
                let mut interval =
                    tokio::time::interval(Duration::from_secs(prober.probe_interval_secs()));

                loop {
                    interval.tick().await;
                    let result = prober
                        .probe_tcp(&node_id, &host, port, source_ip.clone())
                        .await;
                    if tx.send(result).is_err() {
                        break; // receiver dropped — engine shut down
                    }
                }
            });

            handles.push(handle);
        }

        // ── Build a (node_id → standby_ip) lookup for DNS swaps ───────────────
        let standby_map: HashMap<String, String> = self
            .nodes
            .iter()
            .map(|n| (n.node_id.clone(), n.standby_ip.clone()))
            .collect();

        let sm = self.state_machine.clone();
        let dns_swap = self.dns_swap.clone();
        let stats = self.health_stats.clone();
        let mut rx = self.result_rx.take().expect("result_rx missing");

        // ── Dispatch loop ──────────────────────────────────────────────────────
        let dispatch = tokio::spawn(async move {
            while let Some(probe_result) = rx.recv().await {
                let node_id = probe_result.node_id.clone();
                let success = probe_result.success;

                // Update rolling health statistics
                let packet_loss = {
                    let mut stats_guard = stats.lock().await;
                    let stat = stats_guard
                        .entry(node_id.clone())
                        .or_insert_with(|| NodeHealthStats::new(node_id.clone()));
                    stat.update(&probe_result);

                    // Calculate packet loss (1.0 - success_rate)
                    Some(1.0 - stat.success_rate)
                };

                // Drive the state machine
                let events = {
                    let mut sm_guard = sm.lock().await;
                    sm_guard.process_probe_result(&node_id, success, packet_loss)
                };

                // React to emitted events
                for event in events {
                    log::info!(
                        "[failover] {:?}: {} → {:?}  reason={}",
                        event.event_type,
                        event.from_node,
                        event.to_node,
                        event.reason
                    );

                    if matches!(event.event_type, FailoverEventType::TrafficSwitched) {
                        if let Some(ref cf) = dns_swap {
                            if let Some(standby_ip) = standby_map.get(&node_id) {
                                let cf = cf.clone();
                                let ip = standby_ip.clone();
                                tokio::spawn(async move {
                                    if let Err(e) = cf.swap_to_ip(&ip).await {
                                        log::error!("[failover] Cloudflare DNS swap failed: {}", e);
                                    }
                                });
                            }
                        }
                    }
                }
            }
        });

        handles.push(dispatch);
        handles
    }

    /// Snapshot of per-node health statistics.
    pub async fn health_snapshot(&self) -> HashMap<String, NodeHealthStats> {
        self.health_stats.lock().await.clone()
    }

    /// Current active node according to the state machine.
    pub async fn active_node(&self) -> Option<String> {
        self.state_machine.lock().await.active_node.clone()
    }
}

// ─────────────── Extension: expose probe_interval_secs on ActiveProber ────────

/// Thin extension so the orchestrator can read the configured interval.
trait ProberExt {
    fn probe_interval_secs(&self) -> u64;
}

impl ProberExt for ActiveProber {
    fn probe_interval_secs(&self) -> u64 {
        // ActiveProber stores probe_interval as Duration; we expose secs here.
        // Matching the constructor: ActiveProber::new(timeout_ms, interval_secs)
        // The field is private so we re-derive from the Duration stored internally.
        // We store it as a secs value by storing the Duration in the struct —
        // this access calls the Duration's as_secs() via the existing accessor.
        self.probe_interval().as_secs()
    }
}

// ──────────────────────────────────── Tests ───────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_failover_on_threshold() {
        let mut sm = AutoFailover::new(FailoverConfig::default());
        sm.register_node("node1".to_string(), true);
        sm.register_node("node2".to_string(), false);

        let e1 = sm.process_probe_result("node1", false, None);
        assert!(e1.is_empty(), "below threshold — no events yet");

        let e2 = sm.process_probe_result("node1", false, None);
        assert!(e2.is_empty());

        let e3 = sm.process_probe_result("node1", false, None);
        // At threshold — NodeFailed + TrafficSwitched
        assert!(
            e3.iter()
                .any(|e| matches!(e.event_type, FailoverEventType::NodeFailed))
        );
        assert_eq!(sm.get_active_node(), Some(&"node2".to_string()));
        assert_eq!(sm.get_node_state("node1"), Some(&NodeState::Failed));
    }

    #[test]
    fn test_failover_on_packet_loss() {
        let mut sm = AutoFailover::new(FailoverConfig::default());
        sm.register_node("node1".to_string(), true);
        sm.register_node("node2".to_string(), false);

        // Simulate 25% packet loss (above 20% threshold)
        let events = sm.process_probe_result("node1", true, Some(0.25));

        assert!(
            events
                .iter()
                .any(|e| matches!(e.event_type, FailoverEventType::NodeFailed)),
            "should failover on high packet loss"
        );
        assert!(
            events
                .iter()
                .any(|e| e.reason.contains("Packet loss exceeded")),
            "reason should mention packet loss"
        );
    }

    #[test]
    fn test_recovery_emits_event() {
        let mut sm = AutoFailover::new(FailoverConfig {
            failure_threshold: 2,
            recovery_threshold: 3,
            cooldown_seconds: 0,
            ..Default::default()
        });
        sm.register_node("node1".to_string(), true);
        sm.register_node("node2".to_string(), false);

        sm.process_probe_result("node1", false, None);
        sm.process_probe_result("node1", false, None);

        sm.process_probe_result("node1", true, None);
        sm.process_probe_result("node1", true, None);
        let events = sm.process_probe_result("node1", true, None);

        assert!(
            events
                .iter()
                .any(|e| matches!(e.event_type, FailoverEventType::NodeRecovered)),
            "should emit NodeRecovered after threshold successes"
        );
    }

    #[test]
    fn test_iranian_source_ip_format() {
        let ip = pick_iranian_source_ip("5.34.0.0/16");
        assert!(ip.starts_with("5.34."), "expected 5.34.x.x, got {}", ip);
    }

    #[test]
    fn test_all_iranian_prefixes_produce_valid_ips() {
        for (prefix, _desc) in IRANIAN_PROBE_PREFIXES {
            let ip = pick_iranian_source_ip(prefix);
            assert!(
                ip.chars().filter(|&c| c == '.').count() == 3,
                "invalid IP produced for {}: {}",
                prefix,
                ip
            );
        }
    }
}
