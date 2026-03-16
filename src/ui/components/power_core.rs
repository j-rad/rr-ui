//! Power Core Component
//!
//! A dynamic SVG orb that visualizes the heartbeat of the network engine.
//! Replaces the standard "Connect Switch" with a live, animated state machine.
//!
//! # States
//! - **Idle**: Breathing purple glow with slow ring rotation
//! - **Connecting**: Concentric cyan spin with accelerating dash-offset
//! - **Connected**: Cyan plasma hum with stable inner pulse

use crate::ui::theme;
use dioxus::prelude::*;

/// The connection state of the Power Core orb.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum CoreState {
    /// Engine idle — breathing purple pulse
    #[default]
    Idle,
    /// Establishing connection — concentric cyan spin
    Connecting,
    /// Connection active — plasma hum in cyan
    Connected,
}

impl CoreState {
    /// Primary color for the current state.
    pub fn primary_color(&self) -> &'static str {
        match self {
            CoreState::Idle => theme::COLOR_CYBER_PURPLE,
            CoreState::Connecting => theme::COLOR_ELECTRIC_CYAN,
            CoreState::Connected => theme::COLOR_ELECTRIC_CYAN,
        }
    }

    /// Secondary (fill) color for inner glow.
    pub fn glow_color(&self) -> &'static str {
        match self {
            CoreState::Idle => "rgba(188, 19, 254, 0.15)",
            CoreState::Connecting => "rgba(0, 242, 255, 0.20)",
            CoreState::Connected => "rgba(0, 242, 255, 0.25)",
        }
    }

    /// Animation duration for the outer ring rotation (seconds).
    pub fn outer_ring_duration(&self) -> f32 {
        match self {
            CoreState::Idle => 8.0,
            CoreState::Connecting => 2.0,
            CoreState::Connected => 6.0,
        }
    }

    /// Animation duration for the middle ring rotation (seconds).
    pub fn middle_ring_duration(&self) -> f32 {
        match self {
            CoreState::Idle => 12.0,
            CoreState::Connecting => 1.5,
            CoreState::Connected => 8.0,
        }
    }

    /// Inner pulse animation duration (seconds).
    pub fn inner_pulse_duration(&self) -> f32 {
        match self {
            CoreState::Idle => 3.0,
            CoreState::Connecting => 0.8,
            CoreState::Connected => 2.0,
        }
    }

    /// Dash-array for the outer ring stroke.
    pub fn outer_dash_array(&self) -> &'static str {
        match self {
            CoreState::Idle => "8 12",
            CoreState::Connecting => "4 8",
            CoreState::Connected => "12 6",
        }
    }

    /// Dash-array for the middle ring stroke.
    pub fn middle_dash_array(&self) -> &'static str {
        match self {
            CoreState::Idle => "16 8",
            CoreState::Connecting => "6 10",
            CoreState::Connected => "20 4",
        }
    }

    /// Human-readable label
    pub fn label(&self) -> &'static str {
        match self {
            CoreState::Idle => "IDLE",
            CoreState::Connecting => "CONNECTING",
            CoreState::Connected => "CONNECTED",
        }
    }
}

/// Props for the PowerCore component.
#[derive(Props, Clone, PartialEq)]
pub struct PowerCoreProps {
    /// Current connection state
    pub state: CoreState,

    /// Callback fired when the orb is clicked (toggle connection)
    #[props(default)]
    pub on_toggle: Option<EventHandler<MouseEvent>>,

    /// Protocol health signal (0.0 = unhealthy, 1.0 = perfect)
    #[props(default = 1.0)]
    pub protocol_health: f32,

    /// Size of the orb in pixels
    #[props(default = 200)]
    pub size: u32,
}

/// The Power Core orb — a three-layered SVG state machine.
///
/// Layers:
/// 1. **Outer Ring** — rotating dashed circle visualizing connection cadence
/// 2. **Middle Ring** — maps stroke length to protocol health signal
/// 3. **Inner Core** — pulsing filled circle representing engine heartbeat
#[component]
pub fn PowerCore(props: PowerCoreProps) -> Element {
    let state = props.state;
    let size = props.size;
    let half = size as f32 / 2.0;
    let outer_r = half - 8.0;
    let middle_r = half - 24.0;
    let inner_r = half - 42.0;

    let primary = state.primary_color();
    let glow = state.glow_color();
    let outer_dur = state.outer_ring_duration();
    let middle_dur = state.middle_ring_duration();
    let pulse_dur = state.inner_pulse_duration();
    let outer_dash = state.outer_dash_array();
    let middle_dash = state.middle_dash_array();
    let label = state.label();

    // Protocol health maps to the middle ring stroke opacity
    let health = props.protocol_health.clamp(0.0, 1.0);
    let health_opacity = 0.3 + (health * 0.7);

    // CSS animation keyframes for rotation and pulse
    let animations = format!(
        r#"
        @keyframes edgeray-spin-outer {{ from {{ transform: rotate(0deg); }} to {{ transform: rotate(360deg); }} }}
        @keyframes edgeray-spin-middle {{ from {{ transform: rotate(360deg); }} to {{ transform: rotate(0deg); }} }}
        @keyframes edgeray-pulse {{ 0%,100% {{ opacity: 0.6; r: {inner_r}; }} 50% {{ opacity: 1.0; r: {pulse_r}; }} }}
        "#,
        inner_r = inner_r,
        pulse_r = inner_r + 4.0,
    );

    let cursor_class = if props.on_toggle.is_some() {
        "cursor-pointer"
    } else {
        ""
    };

    rsx! {
        div {
            class: "relative inline-flex flex-col items-center gap-4 select-none {cursor_class}",

            // Inject animation keyframes
            style { {animations} }

            // Ambient glow behind the orb
            div {
                class: "absolute rounded-full blur-3xl {theme::GPU_ACCELERATED}",
                style: "width: {size}px; height: {size}px; background: {glow}; top: 0; left: 0;",
            }

            svg {
                width: "{size}",
                height: "{size}",
                view_box: "0 0 {size} {size}",
                onclick: move |evt| {
                    if let Some(handler) = &props.on_toggle {
                        handler.call(evt);
                    }
                },

                // ── Layer 1: Outer Ring ──
                circle {
                    cx: "{half}",
                    cy: "{half}",
                    r: "{outer_r}",
                    fill: "none",
                    stroke: "{primary}",
                    stroke_width: "1.5",
                    stroke_dasharray: "{outer_dash}",
                    stroke_linecap: "round",
                    opacity: "0.6",
                    style: "transform-origin: {half}px {half}px; animation: edgeray-spin-outer {outer_dur}s linear infinite;",
                }

                // ── Layer 2: Middle Ring (protocol health) ──
                circle {
                    cx: "{half}",
                    cy: "{half}",
                    r: "{middle_r}",
                    fill: "none",
                    stroke: "{primary}",
                    stroke_width: "2",
                    stroke_dasharray: "{middle_dash}",
                    stroke_linecap: "round",
                    opacity: "{health_opacity}",
                    style: "transform-origin: {half}px {half}px; animation: edgeray-spin-middle {middle_dur}s linear infinite;",
                }

                // ── Layer 3: Inner Core (pulse) ──
                circle {
                    cx: "{half}",
                    cy: "{half}",
                    r: "{inner_r}",
                    fill: "{primary}",
                    opacity: "0.8",
                    style: "transform-origin: {half}px {half}px; animation: edgeray-pulse {pulse_dur}s ease-in-out infinite;",
                }

                // ── Center dot ──
                circle {
                    cx: "{half}",
                    cy: "{half}",
                    r: "4",
                    fill: "white",
                    opacity: "0.9",
                }
            }

            // State label
            span {
                class: "{theme::FONT_TELEMETRY} text-xs tracking-[0.3em] uppercase",
                style: "color: {primary};",
                "{label}"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_state_labels() {
        assert_eq!(CoreState::Idle.label(), "IDLE");
        assert_eq!(CoreState::Connecting.label(), "CONNECTING");
        assert_eq!(CoreState::Connected.label(), "CONNECTED");
    }

    #[test]
    fn test_core_state_colors() {
        assert_eq!(CoreState::Idle.primary_color(), theme::COLOR_CYBER_PURPLE);
        assert_eq!(
            CoreState::Connecting.primary_color(),
            theme::COLOR_ELECTRIC_CYAN
        );
        assert_eq!(
            CoreState::Connected.primary_color(),
            theme::COLOR_ELECTRIC_CYAN
        );
    }

    #[test]
    fn test_animation_durations_are_positive() {
        for state in [CoreState::Idle, CoreState::Connecting, CoreState::Connected] {
            assert!(state.outer_ring_duration() > 0.0);
            assert!(state.middle_ring_duration() > 0.0);
            assert!(state.inner_pulse_duration() > 0.0);
        }
    }

    #[test]
    fn test_connecting_is_fastest() {
        assert!(
            CoreState::Connecting.outer_ring_duration() < CoreState::Idle.outer_ring_duration()
        );
        assert!(
            CoreState::Connecting.inner_pulse_duration()
                < CoreState::Connected.inner_pulse_duration()
        );
    }

    #[test]
    fn test_dash_arrays_are_non_empty() {
        for state in [CoreState::Idle, CoreState::Connecting, CoreState::Connected] {
            assert!(!state.outer_dash_array().is_empty());
            assert!(!state.middle_dash_array().is_empty());
        }
    }
}
