//! Mimicry Gauge Component
//!
//! Real-time SVG gauge for ShadowMieru (SMR) traffic-shaping telemetry.
//! Displays PSD entropy score as a ring gauge with supporting stat cards
//! for IAT jitter, decoy fallbacks, and blocked probes.

use crate::domain::models::SmrTelemetry;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct MimicryGaugeProps {
    pub metrics: SmrTelemetry,
    #[props(default = 140)]
    pub gauge_size: i32,
}

#[component]
pub fn MimicryGauge(props: MimicryGaugeProps) -> Element {
    let m = &props.metrics;

    // PSD entropy is 0.0 – 1.0; higher is better (more like the mimicked profile).
    let entropy_pct = (m.current_psd_entropy.clamp(0.0, 1.0) * 100.0) as f64;

    // Colour shifts from red (bad mimicry) → yellow → green (perfect mimicry)
    let ring_color = if entropy_pct >= 80.0 {
        "#10b981" // green-500
    } else if entropy_pct >= 50.0 {
        "#f59e0b" // amber-500
    } else {
        "#ef4444" // red-500
    };

    let label = if entropy_pct >= 80.0 {
        "Stealth"
    } else if entropy_pct >= 50.0 {
        "Moderate"
    } else {
        "Exposed"
    };

    // Arc math: 270° sweep identical to CpuGauge
    let arc_len = 188.5_f64; // circumference of the visible arc
    let dash_offset = arc_len * (1.0 - entropy_pct / 100.0);

    rsx! {
        div { class: "bg-glass-bg backdrop-blur-xl border border-glass-border rounded-2xl p-5 hover:border-emerald-500/20 transition-all duration-300",
            div { class: "flex items-center justify-between mb-4",
                div { class: "text-xs uppercase text-gray-500 tracking-wider font-medium", "SMR Mimicry" }
                span { class: "px-2.5 py-1 rounded-lg border text-[10px] font-bold",
                    style: "color: {ring_color}; border-color: {ring_color}33; background: {ring_color}1a;",
                    "{label}"
                }
            }

            div { class: "flex items-center justify-around gap-4",
                // PSD Entropy ring gauge
                div { class: "relative flex items-center justify-center shrink-0",
                    style: "width: {props.gauge_size}px; height: {props.gauge_size}px;",

                    svg {
                        width: "{props.gauge_size}",
                        height: "{props.gauge_size}",
                        view_box: "0 0 120 120",

                        // Background arc
                        path {
                            d: "M 20 60 A 40 40 0 1 1 100 60",
                            fill: "none",
                            stroke: "#2a2a2a",
                            stroke_width: "8",
                            stroke_linecap: "round",
                        }

                        // Progress arc
                        path {
                            d: "M 20 60 A 40 40 0 1 1 100 60",
                            fill: "none",
                            stroke: "{ring_color}",
                            stroke_width: "8",
                            stroke_linecap: "round",
                            stroke_dasharray: "{arc_len}",
                            stroke_dashoffset: "{dash_offset}",
                            style: "transition: stroke-dashoffset 0.6s cubic-bezier(.4,0,.2,1);",
                        }
                    }

                    // Centre label
                    div { class: "absolute inset-0 flex flex-col items-center justify-center",
                        div { class: "text-2xl font-bold text-white", "{entropy_pct:.0}%" }
                        div { class: "text-[10px] text-gray-500", "PSD Entropy" }
                    }
                }

                // Stat pills
                div { class: "grid grid-cols-1 gap-2.5 flex-1 min-w-0",
                    // IAT Jitter
                    div { class: "flex items-center justify-between p-2.5 bg-black/20 rounded-lg",
                        div { class: "flex items-center gap-2",
                            span { class: "material-symbols-outlined text-sm text-cyan-400", "timer" }
                            span { class: "text-[11px] text-gray-400", "IAT Jitter" }
                        }
                        span { class: "text-sm font-mono font-bold text-cyan-400", "{m.iat_jitter_ms:.1}ms" }
                    }

                    // Decoy Fallbacks
                    div { class: "flex items-center justify-between p-2.5 bg-black/20 rounded-lg",
                        div { class: "flex items-center gap-2",
                            span { class: "material-symbols-outlined text-sm text-amber-400", "shield" }
                            span { class: "text-[11px] text-gray-400", "Decoy Triggers" }
                        }
                        span { class: "text-sm font-mono font-bold text-amber-400", "{m.decoy_fallback_triggers}" }
                    }

                    // Probes Blocked
                    div { class: "flex items-center justify-between p-2.5 bg-black/20 rounded-lg",
                        div { class: "flex items-center gap-2",
                            span { class: "material-symbols-outlined text-sm text-red-400", "block" }
                            span { class: "text-[11px] text-gray-400", "Probes Blocked" }
                        }
                        span { class: "text-sm font-mono font-bold text-red-400", "{m.active_probes_blocked}" }
                    }
                }
            }
        }
    }
}
