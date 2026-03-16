//! Status Indicator Component
//!
//! Reactive sidebar status indicator showing core connectivity state.

use crate::ui::state::{CoreConnectivity, GlobalState};
use dioxus::prelude::*;

/// Status indicator style variants
#[derive(Clone, Copy, PartialEq, Default)]
pub enum StatusVariant {
    #[default]
    Dot,
    Badge,
    Full,
}

#[derive(Props, Clone, PartialEq)]
pub struct StatusIndicatorProps {
    /// Display variant
    #[props(default)]
    pub variant: StatusVariant,
    /// Show text label
    #[props(default = true)]
    pub show_label: bool,
}

/// Get status color classes based on connectivity
fn get_status_classes(status: CoreConnectivity) -> (&'static str, &'static str, &'static str) {
    match status {
        CoreConnectivity::Connected => (
            "bg-green-500",        // dot color
            "text-green-400",      // text color
            "border-green-500/30", // border color
        ),
        CoreConnectivity::TransportError => {
            ("bg-orange-500", "text-orange-400", "border-orange-500/30")
        }
        CoreConnectivity::CoreOffline => ("bg-red-500", "text-red-400", "border-red-500/30"),
    }
}

/// Reactive status indicator for sidebar
#[component]
pub fn StatusIndicator(props: StatusIndicatorProps) -> Element {
    let state = use_context::<GlobalState>();
    let core_status = state.core_status_read()();

    let (dot_color, text_color, border_color) = get_status_classes(core_status);
    let status_text = core_status.status_text();
    let is_connected = core_status.is_connected();

    match props.variant {
        StatusVariant::Dot => {
            rsx! {
                div { class: "flex items-center gap-2",
                    div {
                        class: "w-2 h-2 rounded-full {dot_color}",
                        class: if is_connected { "animate-pulse" } else { "" },
                    }
                    if props.show_label {
                        span { class: "text-xs {text_color}", "{status_text}" }
                    }
                }
            }
        }
        StatusVariant::Badge => {
            rsx! {
                div {
                    class: "inline-flex items-center gap-1.5 px-2 py-1 rounded-full border {border_color} bg-black/20",
                    div {
                        class: "w-1.5 h-1.5 rounded-full {dot_color}",
                        class: if is_connected { "animate-pulse" } else { "" },
                    }
                    if props.show_label {
                        span { class: "text-xs font-medium {text_color}", "{status_text}" }
                    }
                }
            }
        }
        StatusVariant::Full => {
            let icon = match core_status {
                CoreConnectivity::Connected => "check_circle",
                CoreConnectivity::TransportError => "warning",
                CoreConnectivity::CoreOffline => "error",
            };

            rsx! {
                div {
                    class: "flex items-center gap-3 p-3 rounded-lg border {border_color} bg-black/20",
                    span { class: "material-symbols-outlined text-[20px] {text_color}", "{icon}" }
                    div {
                        div { class: "text-sm font-medium {text_color}", "{status_text}" }
                        div { class: "text-xs text-gray-500",
                            match core_status {
                                CoreConnectivity::Connected => "RustRay core is running",
                                CoreConnectivity::TransportError => "Connection issues",
                                CoreConnectivity::CoreOffline => "Core is not responding",
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Compact status for header bar
#[component]
pub fn HeaderStatus() -> Element {
    let state = use_context::<GlobalState>();
    let core_status = state.core_status_read()();
    let server_status = state.server_status_read()();

    let (dot_color, text_color, _) = get_status_classes(core_status);

    rsx! {
        div { class: "flex items-center gap-4",
            // Core status
            div { class: "flex items-center gap-2",
                div {
                    class: "w-2 h-2 rounded-full {dot_color}",
                    class: if core_status.is_connected() { "animate-pulse" } else { "" },
                }
                span { class: "text-xs {text_color}", "{core_status.status_text()}" }
            }

            // CPU indicator (if connected)
            if core_status.is_connected() {
                div { class: "flex items-center gap-1 text-xs",
                    span { class: "text-gray-500", "CPU" }
                    span {
                        class: if server_status.cpu > 80.0 { "text-red-400" } else if server_status.cpu > 50.0 { "text-orange-400" } else { "text-green-400" },
                        "{server_status.cpu:.0}%"
                    }
                }
            }
        }
    }
}

/// Sidebar status section with traffic sparkline
#[component]
pub fn SidebarStatus() -> Element {
    let state = use_context::<GlobalState>();
    let traffic_history = state.traffic_history_read()();

    // Calculate current rates (difference between last two points)
    let (upload_rate, download_rate) = if traffic_history.len() >= 2 {
        let last = traffic_history.back().copied().unwrap_or((0, 0));
        let prev = traffic_history
            .iter()
            .rev()
            .nth(1)
            .copied()
            .unwrap_or((0, 0));
        (last.0 - prev.0, last.1 - prev.1)
    } else {
        (0, 0)
    };

    rsx! {
        div { class: "p-4 space-y-3",
            // Status indicator
            StatusIndicator {
                variant: StatusVariant::Badge,
                show_label: true,
            }

            // Traffic mini chart
            if !traffic_history.is_empty() {
                div { class: "space-y-1",
                    div { class: "flex items-center justify-between text-xs",
                        span { class: "text-gray-500", "Traffic" }
                        div { class: "flex items-center gap-2",
                            span { class: "text-blue-400", "↑ {format_bytes_short(upload_rate)}/s" }
                            span { class: "text-green-400", "↓ {format_bytes_short(download_rate)}/s" }
                        }
                    }
                    div { class: "h-8 bg-bg-tertiary rounded overflow-hidden",
                        crate::ui::components::sparkline::MiniSparkline {
                            data: traffic_history,
                        }
                    }
                }
            }
        }
    }
}

/// Format bytes to short string
fn format_bytes_short(bytes: i64) -> String {
    let abs_bytes = bytes.abs();
    const KB: i64 = 1024;
    const MB: i64 = KB * 1024;

    if abs_bytes >= MB {
        format!("{:.1}M", abs_bytes as f64 / MB as f64)
    } else if abs_bytes >= KB {
        format!("{:.0}K", abs_bytes as f64 / KB as f64)
    } else {
        format!("{}", abs_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes_short() {
        assert_eq!(format_bytes_short(512), "512");
        assert_eq!(format_bytes_short(1024), "1K");
        assert_eq!(format_bytes_short(1024 * 1024), "1.0M");
    }

    #[test]
    fn test_get_status_classes() {
        let (dot, text, border) = get_status_classes(CoreConnectivity::Connected);
        assert!(dot.contains("green"));
        assert!(text.contains("green"));
        assert!(border.contains("green"));
    }
}
