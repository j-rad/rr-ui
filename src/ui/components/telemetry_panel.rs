//! Telemetry Panel Component
//!
//! A 3-column dashboard layout using GlassCards to display real-time
//! telemetry data: throughput, latency, and connection health.
//! Integrates the PowerCore orb and InteractiveSparkline components.

use crate::ui::components::glass_card::GlassCard;
use crate::ui::components::interactive_sparkline::{InteractiveSparkline, SparklineBuffer};
use crate::ui::components::power_core::{CoreState, PowerCore};
use crate::ui::theme;
use dioxus::prelude::*;

/// Props for the TelemetryPanel component.
#[derive(Props, Clone, PartialEq)]
pub struct TelemetryPanelProps {
    /// Current Power Core connection state
    #[props(default)]
    pub core_state: CoreState,

    /// Throughput sparkline data (bits per second)
    pub throughput_buffer: SparklineBuffer,

    /// Latency sparkline data (milliseconds)
    pub latency_buffer: SparklineBuffer,

    /// Current throughput value formatted for display
    #[props(default)]
    pub throughput_value: Option<String>,

    /// Current latency value formatted for display
    #[props(default)]
    pub latency_value: Option<String>,

    /// Protocol health signal (0.0 – 1.0)
    #[props(default = 1.0)]
    pub protocol_health: f32,

    /// Callback for Power Core toggle
    #[props(default)]
    pub on_toggle: Option<EventHandler<MouseEvent>>,

    /// Active protocol name (e.g. "REALITY", "VMess", "Fragment")
    #[props(default)]
    pub protocol_name: Option<String>,

    /// Connected server name
    #[props(default)]
    pub server_name: Option<String>,

    /// Connection uptime as formatted string
    #[props(default)]
    pub uptime: Option<String>,
}

/// Three-column telemetry dashboard.
///
/// Layout:
/// ```text
/// ┌─────────────┬─────────────┬─────────────┐
/// │  Throughput  │  Power Core │   Latency   │
/// │  Sparkline   │    Orb      │  Sparkline  │
/// └─────────────┴─────────────┴─────────────┘
/// ```
#[component]
pub fn TelemetryPanel(props: TelemetryPanelProps) -> Element {
    let throughput_color = theme::COLOR_ELECTRIC_CYAN;
    let latency_color = theme::COLOR_EMERALD;

    rsx! {
        div { class: "grid grid-cols-1 lg:grid-cols-3 gap-4 {theme::FONT_GENERAL}",

            // ── Column 1: Throughput ──
            GlassCard {
                title: "THROUGHPUT".to_string(),
                class: "min-h-[180px]".to_string(),

                InteractiveSparkline {
                    buffer: props.throughput_buffer.clone(),
                    width: 240,
                    height: 64,
                    stroke_color: throughput_color.to_string(),
                    gradient_start: "rgba(0, 242, 255, 0.03)".to_string(),
                    gradient_end: "rgba(0, 242, 255, 0.20)".to_string(),
                    label: "BANDWIDTH".to_string(),
                    current_value: props.throughput_value.clone(),
                    unit: "Mbps".to_string(),
                }
            }

            // ── Column 2: Power Core ──
            GlassCard {
                class: "min-h-[180px] flex items-center justify-center".to_string(),
                specular: false,

                div { class: "flex flex-col items-center gap-3",
                    PowerCore {
                        state: props.core_state,
                        on_toggle: props.on_toggle.clone(),
                        protocol_health: props.protocol_health,
                        size: 140,
                    }

                    // Connection metadata
                    div { class: "text-center space-y-1",
                        if let Some(ref proto) = props.protocol_name {
                            div { class: "{theme::FONT_TELEMETRY} text-xs text-white/50 tracking-widest",
                                "{proto}"
                            }
                        }
                        if let Some(ref server) = props.server_name {
                            div { class: "text-xs text-white/40", "{server}" }
                        }
                        if let Some(ref uptime) = props.uptime {
                            div { class: "{theme::FONT_TELEMETRY} text-xs text-white/30",
                                "⏱ {uptime}"
                            }
                        }
                    }
                }
            }

            // ── Column 3: Latency ──
            GlassCard {
                title: "LATENCY".to_string(),
                class: "min-h-[180px]".to_string(),

                InteractiveSparkline {
                    buffer: props.latency_buffer.clone(),
                    width: 240,
                    height: 64,
                    stroke_color: latency_color.to_string(),
                    gradient_start: "rgba(16, 185, 129, 0.03)".to_string(),
                    gradient_end: "rgba(16, 185, 129, 0.20)".to_string(),
                    label: "RTT".to_string(),
                    current_value: props.latency_value.clone(),
                    unit: "ms".to_string(),
                }
            }
        }
    }
}
