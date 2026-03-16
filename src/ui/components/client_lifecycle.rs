//! Client Lifecycle Manager Component
//!
//! Manages client lifecycle including:
//! - IP limit tracking and enforcement
//! - Traffic reset scheduling
//! - Expiration handling and auto-disable

use crate::domain::models::Client;
use dioxus::prelude::*;

/// Client status based on limits and expiration
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ClientStatus {
    Active,
    LimitReached,
    Expired,
    Disabled,
}

impl ClientStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Active => "Active",
            Self::LimitReached => "Limit Reached",
            Self::Expired => "Expired",
            Self::Disabled => "Disabled",
        }
    }

    pub fn color_class(&self) -> &'static str {
        match self {
            Self::Active => "text-green-400",
            Self::LimitReached => "text-orange-400",
            Self::Expired => "text-red-400",
            Self::Disabled => "text-gray-500",
        }
    }

    pub fn bg_class(&self) -> &'static str {
        match self {
            Self::Active => "bg-green-500/20",
            Self::LimitReached => "bg-orange-500/20",
            Self::Expired => "bg-red-500/20",
            Self::Disabled => "bg-gray-500/20",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Active => "check_circle",
            Self::LimitReached => "block",
            Self::Expired => "schedule",
            Self::Disabled => "cancel",
        }
    }
}

/// Determine client status from data
pub fn get_client_status(client: &Client, current_time_ms: i64) -> ClientStatus {
    // Check if explicitly disabled
    if !client.enable {
        return ClientStatus::Disabled;
    }

    // Check expiration
    if client.expiry_time > 0 && current_time_ms > client.expiry_time {
        return ClientStatus::Expired;
    }

    // Check traffic limit
    let total = client.total_flow_limit;
    let up = client.up;
    let down = client.down;
    if total > 0 {
        let total_bytes = (total * 1024 * 1024 * 1024) as i64; // Convert GB to bytes
        if up + down >= total_bytes {
            return ClientStatus::LimitReached;
        }
    }

    ClientStatus::Active
}

/// Format bytes to human-readable string
fn format_bytes(bytes: i64) -> String {
    const GB: i64 = 1024 * 1024 * 1024;
    const MB: i64 = 1024 * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.0} MB", bytes as f64 / MB as f64)
    } else {
        format!("{:.0} KB", bytes as f64 / 1024.0)
    }
}

/// Format timestamp to relative time
fn format_relative_time(timestamp_ms: i64, current_time_ms: i64) -> String {
    let diff_ms = timestamp_ms - current_time_ms;
    let diff_days = diff_ms / (1000 * 60 * 60 * 24);

    if diff_days < 0 {
        format!("{} days ago", -diff_days)
    } else if diff_days == 0 {
        "Today".to_string()
    } else if diff_days == 1 {
        "Tomorrow".to_string()
    } else {
        format!("in {} days", diff_days)
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ClientLifecycleCardProps {
    pub client: Client,
    /// Current time in milliseconds
    pub current_time_ms: i64,
    /// Handler for status change actions
    #[props(default)]
    pub on_action: Option<EventHandler<ClientAction>>,
}

#[derive(Clone, PartialEq)]
pub enum ClientAction {
    Enable(String),
    Disable(String),
    ResetTraffic(String),
    ExtendExpiry(String, i64),
}

/// Individual client lifecycle card
#[component]
pub fn ClientLifecycleCard(props: ClientLifecycleCardProps) -> Element {
    let client = &props.client;
    let status = get_client_status(client, props.current_time_ms);

    let email = client
        .email
        .as_ref()
        .map(|s| s.to_string())
        .unwrap_or_default();
    let up = client.up;
    let down = client.down;
    let total = if client.total_flow_limit > 0 {
        client.total_flow_limit * 1024 * 1024 * 1024
    } else {
        0
    };
    let used_percent = if total > 0 {
        ((up + down) as f64 / total as f64 * 100.0).min(100.0)
    } else {
        0.0
    };

    let expiry_text = if client.expiry_time > 0 {
        format_relative_time(client.expiry_time, props.current_time_ms)
    } else {
        "Never".to_string()
    };

    let handle_toggle = {
        let email = email.clone();
        let on_action = props.on_action.clone();
        let current_enabled = client.enable;
        move |_| {
            if let Some(ref handler) = on_action {
                if current_enabled {
                    handler.call(ClientAction::Disable(email.clone()));
                } else {
                    handler.call(ClientAction::Enable(email.clone()));
                }
            }
        }
    };

    let handle_reset = {
        let email = email.clone();
        let on_action = props.on_action.clone();
        move |_| {
            if let Some(ref handler) = on_action {
                handler.call(ClientAction::ResetTraffic(email.clone()));
            }
        }
    };

    let total_str = if total > 0 {
        format_bytes(total as i64)
    } else {
        "∞".to_string()
    };

    rsx! {
        div { class: "p-4 bg-bg-secondary rounded-lg border border-border hover:border-primary/30 transition-colors",
            // Header
            div { class: "flex items-center justify-between mb-3",
                div { class: "flex items-center gap-3",
                    div { class: "w-10 h-10 rounded-full bg-primary/20 flex items-center justify-center",
                        span { class: "text-primary text-sm font-bold",
                            "{email.chars().next().unwrap_or('?').to_uppercase()}"
                        }
                    }
                    div {
                        div { class: "text-white font-medium", "{email}" }
                        div { class: "text-xs text-gray-500", "Expires: {expiry_text}" }
                    }
                }
                // Status badge
                div {
                    class: "flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium {status.bg_class()} {status.color_class()}",
                    span { class: "material-symbols-outlined text-[14px]", "{status.icon()}" }
                    "{status.label()}"
                }
            }

            // Traffic progress
            div { class: "mb-3",
                div { class: "flex justify-between text-xs text-gray-400 mb-1",
                    span { "Traffic Used" }
                    span { "{format_bytes(up + down)} / {total_str}" }
                }
                div { class: "h-2 bg-bg-tertiary rounded-full overflow-hidden",
                    div {
                        class: "h-full transition-all duration-300",
                        class: if used_percent > 90.0 { "bg-red-500" } else if used_percent > 70.0 { "bg-orange-400" } else { "bg-primary" },
                        style: "width: {used_percent:.0}%",
                    }
                }
            }

            // Stats row
            div { class: "grid grid-cols-3 gap-2 text-center text-xs mb-3",
                div { class: "p-2 bg-bg-tertiary rounded",
                    div { class: "text-gray-500", "↑ Upload" }
                    div { class: "text-white font-medium", "{format_bytes(up)}" }
                }
                div { class: "p-2 bg-bg-tertiary rounded",
                    div { class: "text-gray-500", "↓ Download" }
                    div { class: "text-white font-medium", "{format_bytes(down)}" }
                }
                div { class: "p-2 bg-bg-tertiary rounded",
                    div { class: "text-gray-500", "IP Limit" }
                    div { class: "text-white font-medium", "{client.limit_ip.unwrap_or(0)}" }
                }
            }

            // Actions
            div { class: "flex gap-2",
                button {
                    class: "flex-1 py-2 text-xs font-medium rounded transition-colors",
                    class: if client.enable { "bg-red-500/20 text-red-400 hover:bg-red-500/30" } else { "bg-green-500/20 text-green-400 hover:bg-green-500/30" },
                    onclick: handle_toggle,
                    if client.enable { "Disable" } else { "Enable" }
                }
                button {
                    class: "flex-1 py-2 text-xs font-medium rounded bg-orange-500/20 text-orange-400 hover:bg-orange-500/30 transition-colors",
                    onclick: handle_reset,
                    "Reset Traffic"
                }
            }
        }
    }
}

/// Traffic reset scheduler configuration
#[derive(Props, Clone, PartialEq)]
pub struct TrafficResetSchedulerProps {
    /// Day of month for reset (1-31)
    pub reset_day: Signal<i32>,
    /// Called when schedule is saved
    #[props(default)]
    pub on_save: Option<EventHandler<i32>>,
}

#[component]
pub fn TrafficResetScheduler(props: TrafficResetSchedulerProps) -> Element {
    let mut reset_day = props.reset_day;

    let handle_save = move |_| {
        if let Some(ref handler) = props.on_save {
            handler.call(reset_day());
        }
    };

    rsx! {
        div { class: "p-4 bg-bg-secondary rounded-lg border border-border",
            h3 { class: "text-lg font-semibold text-white mb-4 flex items-center gap-2",
                span { class: "material-symbols-outlined text-primary", "event_repeat" }
                "Traffic Reset Schedule"
            }

            div { class: "space-y-4",
                div {
                    label { class: "block text-sm font-medium text-gray-300 mb-2",
                        "Reset Day of Month"
                    }
                    div { class: "flex items-center gap-3",
                        input {
                            r#type: "number",
                            class: "w-24 h-10 px-3 bg-bg-tertiary border border-border rounded-lg text-white text-center focus:border-primary focus:outline-none",
                            min: "1",
                            max: "31",
                            value: "{reset_day}",
                            oninput: move |evt| {
                                if let Ok(day) = evt.value().parse::<i32>() {
                                    reset_day.set(day.clamp(1, 31));
                                }
                            },
                        }
                        span { class: "text-gray-400 text-sm", "of each month" }
                    }
                    p { class: "text-xs text-gray-500 mt-2",
                        "All client traffic counters will be reset to 0 on this day."
                    }
                }

                button {
                    class: "w-full py-2.5 bg-primary hover:bg-primary-hover text-white font-medium rounded-lg transition-colors",
                    onclick: handle_save,
                    "Save Schedule"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_status_active() {
        let client = Client {
            enable: true,
            expiry_time: i64::MAX,
            total_flow_limit: 100,
            up: 1024,
            down: 1024,
            ..Default::default()
        };
        assert_eq!(get_client_status(&client, 0), ClientStatus::Active);
    }

    #[test]
    fn test_client_status_disabled() {
        let client = Client {
            enable: false,
            ..Default::default()
        };
        assert_eq!(get_client_status(&client, 0), ClientStatus::Disabled);
    }

    #[test]
    fn test_format_bytes() {
        assert!(format_bytes(1024 * 1024 * 1024).contains("GB"));
        assert!(format_bytes(1024 * 1024).contains("MB"));
    }
}
