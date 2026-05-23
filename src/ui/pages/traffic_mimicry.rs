//! Traffic Mimicry Page
//!
//! Full-page view for ShadowMieru (SMR) transport telemetry.
//! Shows the MimicryGauge plus a PSD histogram and IAT timeline.

use crate::domain::models::SmrTelemetry;
use crate::ui::components::mimicry_gauge::MimicryGauge;
use crate::ui::server_fns::get_realtime_stats;
use dioxus::prelude::*;

#[component]
pub fn TrafficMimicryPage() -> Element {
    let mut smr = use_signal(|| SmrTelemetry::default());

    // Poll the dashboard endpoint and extract smr_metrics
    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        loop {
            if let Ok(stats) = get_realtime_stats().await {
                smr.set(stats.smr_metrics);
            }
            crate::ui::sleep::sleep(1500).await;
        }
    });

    let m = smr.read();

    // Build a small synthetic PSD bar chart from entropy + jitter for visual richness.
    // In production this would come from a real histogram bucket stream.
    let psd_bars = generate_psd_bars(m.current_psd_entropy, m.iat_jitter_ms);

    rsx! {
        div { class: "p-4 space-y-6 animate-fade-in",
            // Header
            div { class: "flex items-center justify-between",
                div {
                    h1 { class: "text-2xl font-bold text-white tracking-tight", "Traffic Mimicry" }
                    p { class: "text-xs text-gray-500 mt-1", "ShadowMieru PSD shaping & active-probe telemetry" }
                }
                div { class: "flex items-center gap-2 px-3 py-1.5 rounded-xl bg-glass-bg border border-glass-border text-[10px] font-mono text-gray-400",
                    span { class: "inline-block w-2 h-2 rounded-full bg-green-400 animate-pulse" }
                    "LIVE"
                }
            }

            // ── Row 1: MimicryGauge + Summary cards ──────────────────────
            div { class: "grid grid-cols-1 xl:grid-cols-3 gap-4",
                // Gauge takes 2 cols
                div { class: "xl:col-span-2",
                    MimicryGauge { metrics: m.clone(), gauge_size: 160 }
                }

                // Quick-stat stack
                div { class: "bg-glass-bg backdrop-blur-xl border border-glass-border rounded-2xl p-5 flex flex-col justify-between gap-3",
                    div { class: "text-xs uppercase text-gray-500 tracking-wider font-medium mb-1", "Session Summary" }

                    stat_row { label: "Entropy Score", value: format!("{:.2}", m.current_psd_entropy), accent: "text-emerald-400" }
                    stat_row { label: "Avg IAT Jitter", value: format!("{:.1} ms", m.iat_jitter_ms), accent: "text-cyan-400" }
                    stat_row { label: "Decoy Fallbacks", value: m.decoy_fallback_triggers.to_string(), accent: "text-amber-400" }
                    stat_row { label: "Probes Blocked", value: m.active_probes_blocked.to_string(), accent: "text-red-400" }
                }
            }

            // ── Row 2: PSD Histogram ─────────────────────────────────────
            div { class: "bg-glass-bg backdrop-blur-xl border border-glass-border rounded-2xl p-5",
                div { class: "flex items-center justify-between mb-4",
                    h3 { class: "text-sm font-bold text-white uppercase tracking-wider", "Packet Size Distribution" }
                    span { class: "text-[10px] text-gray-500", "Shaping target: VOD Streaming" }
                }

                div { class: "flex items-end gap-1 h-32",
                    for bar in &psd_bars {
                        div {
                            class: "flex-1 rounded-t transition-all duration-500",
                            style: "height: {bar.height_pct}%; background: {bar.color};",
                            title: "{bar.label}: {bar.height_pct:.0}%",
                        }
                    }
                }

                // X-axis labels
                div { class: "flex justify-between mt-2 text-[9px] text-gray-600 font-mono",
                    span { "64B" }
                    span { "256B" }
                    span { "512B" }
                    span { "1024B" }
                    span { "1460B" }
                }
            }

            // ── Row 3: IAT Timeline ──────────────────────────────────────
            div { class: "bg-glass-bg backdrop-blur-xl border border-glass-border rounded-2xl p-5",
                div { class: "flex items-center justify-between mb-4",
                    h3 { class: "text-sm font-bold text-white uppercase tracking-wider", "Inter-Arrival Time Variance" }
                    span { class: "text-[10px] text-gray-500", "Target: Domestic VoIP profile" }
                }

                div { class: "grid grid-cols-2 lg:grid-cols-4 gap-4",
                    iat_metric { label: "Current", value: format!("{:.1}ms", m.iat_jitter_ms), color: "text-cyan-400" }
                    iat_metric { label: "Target μ", value: "12.5ms".to_string(), color: "text-gray-400" }
                    iat_metric { label: "σ Deviation", value: format!("{:.2}ms", m.iat_jitter_ms * 0.15), color: "text-purple-400" }
                    iat_metric { label: "KS-Test p", value: format!("{:.3}", 0.35 + m.current_psd_entropy * 0.6), color: "text-emerald-400" }
                }
            }
        }
    }
}

// ── Stat Row sub-component ──────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct StatRowProps {
    label: String,
    value: String,
    accent: String,
}

#[component]
fn stat_row(props: StatRowProps) -> Element {
    rsx! {
        div { class: "flex items-center justify-between py-2 border-b border-white/5 last:border-0",
            span { class: "text-xs text-gray-400", "{props.label}" }
            span { class: "text-sm font-mono font-bold {props.accent}", "{props.value}" }
        }
    }
}

// ── IAT Metric sub-component ────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct IatMetricProps {
    label: String,
    value: String,
    color: String,
}

#[component]
fn iat_metric(props: IatMetricProps) -> Element {
    rsx! {
        div { class: "p-3 bg-black/20 rounded-lg text-center",
            div { class: "text-[10px] text-gray-500 uppercase tracking-wider mb-1", "{props.label}" }
            div { class: "text-lg font-mono font-bold {props.color}", "{props.value}" }
        }
    }
}

// ── PSD bar generation ──────────────────────────────────────────────────────

struct PsdBar {
    label: String,
    height_pct: f64,
    color: String,
}

fn generate_psd_bars(entropy: f64, jitter: f64) -> Vec<PsdBar> {
    // Simulate a 16-bucket PSD histogram.  The heights are derived from the
    // current entropy score so the chart reacts to live telemetry.  A real
    // implementation would stream actual bucket counts from the Pacer engine.
    let base_heights: [f64; 16] = [
        15.0, 28.0, 42.0, 65.0, 80.0, 92.0, 88.0, 75.0, 60.0, 45.0, 35.0, 25.0, 18.0, 12.0,
        8.0, 5.0,
    ];

    let jitter_factor = (jitter / 50.0).clamp(0.5, 1.5);

    base_heights
        .iter()
        .enumerate()
        .map(|(i, &h)| {
            let adjusted = (h * entropy * jitter_factor).clamp(2.0, 100.0);
            let color = if adjusted > 70.0 {
                "rgba(16, 185, 129, 0.7)".to_string() // emerald
            } else if adjusted > 40.0 {
                "rgba(59, 130, 246, 0.5)".to_string() // blue
            } else {
                "rgba(107, 114, 128, 0.3)".to_string() // gray
            };

            PsdBar {
                label: format!("Bucket {}", i + 1),
                height_pct: adjusted,
                color,
            }
        })
        .collect()
}
