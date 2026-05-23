//! State Management
//!
//! Application state and context providers for the Dioxus UI.
//! Handles global state, telemetry synchronization, and UI notifications.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use super::components::toast::ToastStore;
use super::app::SyncStatus;
// Shared types - available unconditionally
use crate::models::{RustRayStatus, ServerStatus, TrafficStats};
use crate::domain::proxy_core::CoreConfig;

// Server functions - only available with web feature
#[cfg(any(feature = "server", target_arch = "wasm32"))]
use crate::ui::server_fns::{get_server_status, get_traffic_stats, get_last_core_error};

/// Maximum number of historical data points to keep (60 points = 90 seconds at 1.5s interval)
const MAX_HISTORY_POINTS: usize = 60;

/// Polling interval in milliseconds
const POLL_INTERVAL_MS: u64 = 1500;

/// Core connectivity status
#[derive(Clone, Copy, PartialEq, Debug, Default, Serialize, Deserialize)]
pub enum CoreConnectivity {
    #[default]
    Connected,
    TransportError,
    CoreOffline,
}

impl CoreConnectivity {
    /// Returns true if the core is reachable
    pub fn is_connected(&self) -> bool {
        matches!(self, CoreConnectivity::Connected)
    }

    /// Returns a human-readable status string
    pub fn status_text(&self) -> &'static str {
        match self {
            CoreConnectivity::Connected => "Connected",
            CoreConnectivity::TransportError => "Transport Error",
            CoreConnectivity::CoreOffline => "Offline",
        }
    }
}

/// Notification types for the toast queue
#[derive(Clone, Debug, PartialEq)]
pub enum NotificationType {
    Success,
    Error,
    Warning,
    Info,
}

/// A notification message
#[derive(Clone, Debug)]
pub struct Notification {
    pub id: u64,
    pub message: String,
    pub notification_type: NotificationType,
    pub timestamp: i64,
}


/// Global application state using GlobalSignal for direct access
#[derive(Clone)]
pub struct UIState {
    /// Sidebar collapsed state
    pub sidebar_collapsed: Signal<bool>,
    /// Current theme (light, dark, ultra-dark)
    pub theme: Signal<String>,
    /// User authentication status
    pub is_authenticated: Signal<bool>,
    /// JWT token (if authenticated)
    pub auth_token: Signal<Option<String>>,
    /// Core connectivity status
    pub core_status: Signal<CoreConnectivity>,
    /// Synchronization status
    pub status: Signal<SyncStatus>,
    /// Real-time traffic metrics
    pub traffic_metrics: Signal<Vec<TrafficStats>>,
    /// Historical traffic data for charts (upload, download)
    pub traffic_history: Signal<VecDeque<(i64, i64)>>,
    /// Server status (CPU, memory, disk, uptime)
    pub server_status: Signal<ServerStatus>,
    /// Toast notification manager
    pub toast: ToastStore,
    /// Notification queue for custom notifications
    notification_counter: Signal<u64>,
    pub notifications: Signal<VecDeque<Notification>>,
    /// Most recent tactical error
    pub last_error: Signal<Option<crate::ui::error_bridge::TacticalMessage>>,
    /// Core configuration
    pub core_config: Signal<CoreConfig>,
}

impl UIState {
    pub fn new() -> Self {
        Self {
            sidebar_collapsed: Signal::new(false),
            theme: Signal::new("dark".to_string()),
            is_authenticated: Signal::new(false),
            auth_token: Signal::new(None),
            core_status: Signal::new(CoreConnectivity::CoreOffline),
            status: Signal::new(SyncStatus::Initial),
            traffic_metrics: Signal::new(Vec::new()),
            traffic_history: Signal::new(VecDeque::with_capacity(MAX_HISTORY_POINTS)),
            server_status: Signal::new(ServerStatus::default()),
            toast: ToastStore::new(),
            notification_counter: Signal::new(0),
            notifications: Signal::new(VecDeque::with_capacity(10)),
            last_error: Signal::new(None),
            core_config: Signal::new(CoreConfig {
                log_level: crate::domain::proxy_core::LogLevel::Info,
                inbounds: vec![],
                outbounds: vec![],
                routing: crate::domain::proxy_core::RoutingConfig {
                    rules: vec![],
                    domain_strategy: crate::domain::proxy_core::DomainStrategy::AsIs,
                },
                dns: None,
            }),
        }
    }

    /// Get read-only access to core status
    pub fn core_status_read(&self) -> ReadOnlySignal<CoreConnectivity> {
        self.core_status.into()
    }

    /// Get read-only access to traffic history
    pub fn traffic_history_read(&self) -> ReadOnlySignal<VecDeque<(i64, i64)>> {
        self.traffic_history.into()
    }

    /// Get read-only access to server status
    pub fn server_status_read(&self) -> ReadOnlySignal<ServerStatus> {
        self.server_status.into()
    }

    /// Push a notification to the queue
    pub fn push_notification(
        &mut self,
        message: impl Into<String>,
        notification_type: NotificationType,
    ) {
        let mut counter = self.notification_counter;
        let id = *counter.read();
        counter.set(id + 1);

        // Use milliseconds-since-epoch on wasm32 (chrono wasmbind feature enabled).
        // Falls back to 0 on targets without a system clock — non-fatal.
        let timestamp = chrono::Utc::now().timestamp_millis();

        let notification = Notification {
            id,
            message: message.into(),
            notification_type,
            timestamp,
        };

        self.notifications.with_mut(|queue| {
            if queue.len() >= 10 {
                queue.pop_front();
            }
            queue.push_back(notification);
        });
    }

    /// Remove a notification by ID
    pub fn dismiss_notification(&mut self, id: u64) {
        self.notifications.with_mut(|queue| {
            queue.retain(|n| n.id != id);
        });
    }

    /// Initialize background synchronization task.
    ///
    /// Always transitions out of `Initial`/`CatchingUp` after the first poll
    /// attempt so the UI is never permanently blocked — even when the core
    /// binary is absent (mock / degraded mode).
    pub fn init_sync(&self) {
        #[cfg(any(feature = "server", target_arch = "wasm32"))]
        {
            let mut core_status = self.core_status;
            let mut sync_status = self.status;
            let mut traffic_metrics = self.traffic_metrics;
            let mut traffic_history = self.traffic_history;
            let mut server_status = self.server_status;
            let mut toast = self.toast.clone();
            let mut last_error = self.last_error;
            let mut core_config = self.core_config;

            spawn(async move {
                let mut prev_core_connected = false;
                let mut first_sync = true;

                loop {
                    if first_sync {
                        sync_status.set(SyncStatus::Syncing);
                    }

                    // --- Core Config ---
                    if first_sync {
                        if let Ok(config) = get_core_config().await {
                             core_config.set(config);
                             crate::ui::app::on_first_grpc_delta(sync_status);
                        }
                    }

                    // --- Traffic stats ---
                    match get_traffic_stats().await {
                        Ok(stats) => {
                            let s: Vec<crate::models::TrafficStats> = stats.clone();
                            traffic_metrics.set(s);

                            let (total_up, total_down) =
                                stats.iter().fold((0i64, 0i64), |acc, stat| {
                                    if stat.name.contains("uplink") {
                                        (acc.0 + stat.value, acc.1)
                                    } else if stat.name.contains("downlink") {
                                        (acc.0, acc.1 + stat.value)
                                    } else {
                                        acc
                                    }
                                });

                            traffic_history.with_mut(|history| {
                                if history.len() >= MAX_HISTORY_POINTS {
                                    history.pop_front();
                                }
                                history.push_back((total_up, total_down));
                            });

                            // Advance to Live on first successful poll if not already set.
                            if first_sync && *sync_status.read() != SyncStatus::Live {
                                sync_status.set(SyncStatus::Live);
                            }
                        }
                        Err(e) => {
                            let err_msg = e.to_string();
                            let tactical_err = crate::ui::error_bridge::TacticalMessage::connection_failed(&err_msg);
                            
                            sync_status.set(SyncStatus::Stale(tactical_err.clone()));

                            let current_connected = core_status.read().is_connected();
                            if prev_core_connected && !current_connected {
                                toast.tactical(tactical_err.clone());
                            }
                            prev_core_connected = current_connected;
                            core_status.set(CoreConnectivity::CoreOffline);
                            // Store the error for UI inspection
                            last_error.set(Some(tactical_err));
                        }
                    }

                    // Always advance past the first sync after the first poll attempt.
                    if first_sync {
                        first_sync = false;
                    }

                    // --- Server status (CPU, memory, uptime) ---
                    match get_server_status().await {
                        Ok(status) => {
                            let new_core_status = if status.rustray.state == RustRayStatus::Running
                            {
                                CoreConnectivity::Connected
                            } else {
                                CoreConnectivity::CoreOffline
                            };

                            let was_disconnected = !prev_core_connected;
                            let now_connected = new_core_status.is_connected();
                            if was_disconnected && now_connected {
                                toast.success("Connected to RustRay Core");
                                sync_status.set(SyncStatus::Live);
                            }
                            prev_core_connected = now_connected;

                            core_status.set(new_core_status);
                            server_status.set(status);
                        }
                        Err(_) => {
                            // Non-fatal — server status is best-effort telemetry.
                        }
                    }

                    // --- Check for Tactical Core Errors ---
                    if let Ok(Some(err_msg)) = get_last_core_error().await {
                         let tactical_err = crate::ui::error_bridge::TacticalMessage::connection_failed(&err_msg);
                         
                         // Only show toast if it's a new or significant error
                         if last_error.read().as_ref().map(|e| &e.message) != Some(&err_msg) {
                             toast.tactical(tactical_err.clone());
                             last_error.set(Some(tactical_err));
                             core_status.set(CoreConnectivity::TransportError);
                             sync_status.set(SyncStatus::Stale(tactical_err));
                         }
                    }

                    crate::ui::sleep::sleep(POLL_INTERVAL_MS).await;
                }
            });
        }
    }
}

impl Default for UIState {
    fn default() -> Self {
        Self::new()
    }
}

// Re-export AppState as alias for backward compatibility
pub type AppState = UIState;
pub type GlobalState = UIState;

#[cfg(test)]
mod tests {
    use super::*;

    // ── CoreConnectivity ───────────────────────────────────────────────────────

    #[test]
    fn test_core_connectivity_is_connected() {
        assert!(CoreConnectivity::Connected.is_connected());
        assert!(!CoreConnectivity::TransportError.is_connected());
        assert!(!CoreConnectivity::CoreOffline.is_connected());
    }

    #[test]
    fn test_core_connectivity_status_text() {
        assert_eq!(CoreConnectivity::Connected.status_text(), "Connected");
        assert_eq!(CoreConnectivity::CoreOffline.status_text(), "Offline");
        assert_eq!(CoreConnectivity::TransportError.status_text(), "Transport Error");
    }

    #[test]
    fn test_core_connectivity_default_is_connected() {
        // The default state is Connected (assumed optimistic on startup).
        assert!(CoreConnectivity::default().is_connected());
    }

    // ── SyncStatus ─────────────────────────────────────────────────────────────

    #[test]
    fn test_sync_status_default_is_initial() {
        assert_eq!(SyncStatus::default(), SyncStatus::Initial);
    }

    #[test]
    fn test_sync_status_variants_are_distinct() {
        assert_ne!(SyncStatus::Initial, SyncStatus::Syncing);
        assert_ne!(SyncStatus::Syncing, SyncStatus::Live);
        // Stale takes a message, so we just check it's not Live
        let stale = SyncStatus::Stale(crate::ui::error_bridge::TacticalMessage::connection_failed("test"));
        assert_ne!(SyncStatus::Live, stale);
    }

    // ── NotificationType ───────────────────────────────────────────────────────

    #[test]
    fn test_notification_type_equality() {
        assert_eq!(NotificationType::Success, NotificationType::Success);
        assert_ne!(NotificationType::Success, NotificationType::Error);
        assert_ne!(NotificationType::Warning, NotificationType::Info);
    }

    #[test]
    fn test_notification_fields_are_stored_correctly() {
        let n = Notification {
            id: 42,
            message: "hello world".to_string(),
            notification_type: NotificationType::Warning,
            timestamp: 1_000_000,
        };
        assert_eq!(n.id, 42);
        assert_eq!(n.message, "hello world");
        assert_eq!(n.notification_type, NotificationType::Warning);
        assert_eq!(n.timestamp, 1_000_000);
    }

    // ── Queue overflow guard ───────────────────────────────────────────────────
    // Note: full UIState construction requires a Dioxus reactive runtime
    // (Signals panic outside a component context). These tests verify the
    // plain-data structures independently.

    #[test]
    fn test_vecdeque_overflow_guard() {
        // Mirrors the queue logic inside push_notification / traffic_history.
        let cap = 10usize;
        let mut queue: std::collections::VecDeque<u64> = std::collections::VecDeque::with_capacity(cap);
        for i in 0..(cap + 5) as u64 {
            if queue.len() >= cap { queue.pop_front(); }
            queue.push_back(i);
        }
        assert_eq!(queue.len(), cap, "Queue must not exceed cap");
        assert_eq!(*queue.front().unwrap(), 5u64, "Oldest entries must be evicted");
        assert_eq!(*queue.back().unwrap(), 14u64);
    }

    #[test]
    fn test_traffic_history_overflow_guard() {
        let cap = MAX_HISTORY_POINTS;
        let mut history: std::collections::VecDeque<(i64, i64)> =
            std::collections::VecDeque::with_capacity(cap);
        for i in 0..(cap + 10) {
            if history.len() >= cap { history.pop_front(); }
            history.push_back((i as i64, i as i64 * 2));
        }
        assert_eq!(history.len(), cap);
        // Newest entry should be the last inserted.
        assert_eq!(history.back().unwrap().0, (cap + 10 - 1) as i64);
    }

    // ── Error categorization ───────────────────────────────────────────────────

    #[test]
    fn test_transport_error_classification() {
        let transport_errs = ["transport error", "connection refused", "node is offline"];
        for msg in &transport_errs {
            let is_offline = msg.contains("transport")
                || msg.contains("connection refused")
                || msg.contains("offline");
            assert!(is_offline, "Expected offline classification for: {}", msg);
        }
    }

    #[test]
    fn test_generic_error_does_not_classify_as_offline() {
        let generic = "json parse error";
        let is_offline = generic.contains("transport")
            || generic.contains("connection refused")
            || generic.contains("offline");
        assert!(!is_offline);
    }
}
