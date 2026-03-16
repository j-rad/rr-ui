//! State Management
//!
//! Application state and context providers for the Dioxus UI.
//! Handles global state, telemetry synchronization, and UI notifications.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use super::components::toast::ToastStore;
// Shared types - available unconditionally
use crate::models::{RustRayStatus, ServerStatus, TrafficStats};

// Server functions - only available with web feature
#[cfg(feature = "web")]
use crate::ui::server_fns::{get_server_status, get_traffic_stats};

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

/// Global application state
#[derive(Clone)]
pub struct GlobalState {
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
}

impl GlobalState {
    pub fn new() -> Self {
        Self {
            sidebar_collapsed: Signal::new(false),
            theme: Signal::new("dark".to_string()),
            is_authenticated: Signal::new(false),
            auth_token: Signal::new(None),
            core_status: Signal::new(CoreConnectivity::Connected),
            traffic_metrics: Signal::new(Vec::new()),
            traffic_history: Signal::new(VecDeque::with_capacity(MAX_HISTORY_POINTS)),
            server_status: Signal::new(ServerStatus::default()),
            toast: ToastStore::new(),
            notification_counter: Signal::new(0),
            notifications: Signal::new(VecDeque::with_capacity(10)),
        }
    }

    /// Get read-only access to core status (for components that only need to read)
    pub fn core_status_read(&self) -> ReadSignal<CoreConnectivity> {
        self.core_status.into()
    }

    /// Get read-only access to traffic history (for chart components)
    pub fn traffic_history_read(&self) -> ReadSignal<VecDeque<(i64, i64)>> {
        self.traffic_history.into()
    }

    /// Get read-only access to server status
    pub fn server_status_read(&self) -> ReadSignal<ServerStatus> {
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

        let notification = Notification {
            id,
            message: message.into(),
            notification_type,
            timestamp: chrono::Utc::now().timestamp(),
        };

        self.notifications.with_mut(|queue| {
            // Keep max 10 notifications
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

    /// Initialize background synchronization task
    #[cfg(feature = "web")]
    pub fn init_sync(&self) {
        let mut core_status = self.core_status;
        let mut traffic_metrics = self.traffic_metrics;
        let mut traffic_history = self.traffic_history;
        let mut server_status = self.server_status;
        let mut toast = self.toast.clone();

        spawn(async move {
            // Track previous core status to detect changes
            let mut prev_core_connected = true;

            loop {
                // Poll traffic stats from RustRay gRPC
                match get_traffic_stats().await {
                    Ok(stats) => {
                        let s: Vec<crate::models::TrafficStats> = stats.clone();
                        traffic_metrics.set(s);

                        // Aggregate total up/down for history
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

                        // Update history with bounded buffer
                        traffic_history.with_mut(|history| {
                            if history.len() >= MAX_HISTORY_POINTS {
                                history.pop_front();
                            }
                            history.push_back((total_up, total_down));
                        });
                    }
                    Err(e) => {
                        // Handle transport errors gracefully
                        let err_msg: String = e.to_string();
                        let new_status = if err_msg.contains("transport")
                            || err_msg.contains("connection refused")
                            || err_msg.contains("offline")
                        {
                            CoreConnectivity::CoreOffline
                        } else {
                            CoreConnectivity::TransportError
                        };

                        // Only toast on state change to avoid spam
                        let current_connected = core_status.read().is_connected();
                        if prev_core_connected && !current_connected {
                            toast.error("Lost connection to RustRay Core");
                        }
                        prev_core_connected = current_connected;

                        core_status.set(new_status);
                    }
                }

                // Poll server status (CPU, memory, etc.)
                match get_server_status().await {
                    Ok(status) => {
                        // Update core connectivity based on rustray_running field
                        let new_core_status = if status.rustray.state == RustRayStatus::Running {
                            CoreConnectivity::Connected
                        } else {
                            CoreConnectivity::CoreOffline
                        };

                        // Detect reconnection
                        let was_disconnected = !prev_core_connected;
                        let now_connected = new_core_status.is_connected();
                        if was_disconnected && now_connected {
                            toast.success("Connected to RustRay Core");
                        }
                        prev_core_connected = now_connected;

                        core_status.set(new_core_status);
                        server_status.set(status);
                    }
                    Err(_) => {
                        // Server status fetch failed, but don't change core status
                        // Traffic stats are a better indicator of core health
                    }
                }

                // Async sleep using gloo_timers
                crate::ui::sleep::sleep(POLL_INTERVAL_MS as u32 as u64).await;
            }
        });
    }

    /// Stub init_sync for non-web builds
    #[cfg(not(feature = "web"))]
    pub fn init_sync(&self) {
        // No-op on server-only builds
    }
}

impl Default for GlobalState {
    fn default() -> Self {
        Self::new()
    }
}

// Re-export AppState as alias for backward compatibility
pub type AppState = GlobalState;

#[cfg(test)]
mod tests {
    use super::*;

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
    }

    #[test]
    fn test_notification_type() {
        let notif = Notification {
            id: 1,
            message: "Test".to_string(),
            notification_type: NotificationType::Success,
            timestamp: 0,
        };
        assert_eq!(notif.notification_type, NotificationType::Success);
    }
}
