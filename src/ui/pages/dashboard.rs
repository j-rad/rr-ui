//! Bento Dashboard Page
//!
//! Glassmorphic Bento-style grid layout with real-time telemetry.
//! Replaces the legacy NOC table with a modern card-based dashboard.

use crate::domain::models::{ActiveConnection, DashboardStats, DiscoveryState, NodeHealth};
use crate::ui::components::card::Card;
use crate::ui::components::cpu_gauge::CpuGauge;
use crate::ui::components::memory_progress::MemoryProgress;
use crate::ui::components::sparkline::Sparkline;
use crate::ui::server_fns::get_realtime_stats;
use dioxus::prelude::*;
use std::collections::{HashMap, VecDeque};

// ═══════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn status_color_class(status: &DiscoveryState) -> &'static str {
    match status {
        DiscoveryState::Idle => "text-gray-400 bg-gray-400/10 border-gray-400/20",
        DiscoveryState::Scanning => "text-blue-400 bg-blue-400/10 border-blue-400/20 animate-pulse",
        DiscoveryState::ReVerifying => "text-yellow-400 bg-yellow-400/10 border-yellow-400/20",
    }
}

fn rtt_color(rtt: f64) -> &'static str {
    if rtt > 200.0 {
        "text-red-400"
    } else if rtt > 100.0 {
        "text-yellow-400"
    } else {
        "text-green-400"
    }
}

// ═══════════════════════════════════════════════════════════════════
// Dashboard
// ═══════════════════════════════════════════════════════════════════

#[component]
pub fn DashboardPage() -> Element {
    let mut stats = use_signal(|| DashboardStats::default());
    let mut conn_history = use_signal(|| HashMap::<String, VecDeque<(i64, i64)>>::new());
    let mut global_traffic_history = use_signal(|| VecDeque::<(i64, i64)>::with_capacity(60));

    // Polling coroutine
    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        loop {
            if let Ok(new_stats) = get_realtime_stats().await {
                // Aggregate global traffic for the large sparkline
                let (total_up, total_down) = new_stats
                    .active_connections
                    .iter()
                    .fold((0i64, 0i64), |(u, d), c| {
                        (u + c.upload_bytes as i64, d + c.download_bytes as i64)
                    });
                global_traffic_history.with_mut(|h| {
                    if h.len() >= 60 {
                        h.pop_front();
                    }
                    h.push_back((total_up, total_down));
                });

                // Per-connection history
                conn_history.with_mut(|history| {
                    let mut seen_ids = Vec::new();
                    for conn in &new_stats.active_connections {
                        seen_ids.push(conn.id.clone());
                        let deque = history
                            .entry(conn.id.clone())
                            .or_insert_with(|| VecDeque::with_capacity(60));
                        if deque.len() >= 30 {
                            deque.pop_front();
                        }
                        deque.push_back((conn.upload_bytes as i64, conn.download_bytes as i64));
                    }
                    history.retain(|k, _| seen_ids.contains(k));
                });

                stats.set(new_stats);
            }
            crate::ui::sleep::sleep(1000).await;
        }
    });

    let s = stats.read();
    let health = s.node_health.clone().unwrap_or_default();
    let mesh = s.mesh_stats.clone();
    let history_map = conn_history.read();
    let global_history = global_traffic_history.read();

    // Aggregate totals for top cards
    let (total_up, total_down) = s.active_connections.iter().fold((0u64, 0u64), |(u, d), c| {
        (u + c.upload_bytes, d + c.download_bytes)
    });

    // Top 5 connections by total traffic
    let mut top_conns: Vec<&ActiveConnection> = s.active_connections.iter().collect();
    top_conns.sort_by(|a, b| {
        (b.upload_bytes + b.download_bytes).cmp(&(a.upload_bytes + a.download_bytes))
    });
    top_conns.truncate(5);

    rsx! {
        div { class: "p-4 space-y-4 animate-fade-in",

            // ── Row 1: Bento stat cards ─────────────────────────────────
            div { class: "grid grid-cols-1 md:grid-cols-2 xl:grid-cols-4 gap-4",

                // Card 1: Discovery Core
                div { class: "bg-glass-bg backdrop-blur-xl border border-glass-border rounded-2xl p-5 hover:border-cyan-500/20 transition-all duration-300 group",
                    div { class: "flex items-center justify-between mb-4",
                        div { class: "text-xs uppercase text-gray-500 tracking-wider font-medium", "Discovery Core" }
                        span { class: "px-2.5 py-1 rounded-lg border text-[10px] font-bold {status_color_class(&s.discovery_state)}",
                            "{s.discovery_state:?}"
                        }
                    }
                    div { class: "flex items-end justify-between",
                        div {
                            div { class: "text-3xl font-bold text-white animate-counter", "{s.active_connections.len()}" }
                            div { class: "text-[10px] text-gray-500 mt-0.5", "Active Sessions" }
                        }
                        div { class: "text-right",
                            div { class: "text-xs text-gray-500", "Uptime" }
                            div { class: "text-sm font-mono text-gray-400", "14d 2h" }
                        }
                    }
                }

                // Card 2: Mesh Network
                div { class: "bg-glass-bg backdrop-blur-xl border border-glass-border rounded-2xl p-5 hover:border-purple-500/20 transition-all duration-300",
                    div { class: "text-xs uppercase text-gray-500 tracking-wider font-medium mb-4", "Mesh Network" }
                    div { class: "grid grid-cols-2 gap-3",
                        div {
                            div { class: "text-2xl font-bold text-white", "{mesh.total_nodes}" }
                            div { class: "text-[10px] text-gray-500", "Total Nodes" }
                        }
                        div {
                            div { class: "text-2xl font-bold text-green-400", "{mesh.online_nodes}" }
                            div { class: "text-[10px] text-gray-500", "Online" }
                        }
                        div {
                            div { class: "text-2xl font-bold text-blue-400", "{mesh.total_clients}" }
                            div { class: "text-[10px] text-gray-500", "Clients" }
                        }
                        div {
                            div { class: "text-2xl font-bold text-purple-400", "{health.latency_ms:.1}ms" }
                            div { class: "text-[10px] text-gray-500", "Avg Latency" }
                        }
                    }
                }

                // Card 3: System Gauges
                div { class: "bg-glass-bg backdrop-blur-xl border border-glass-border rounded-2xl p-5 hover:border-blue-500/20 transition-all duration-300",
                    div { class: "text-xs uppercase text-gray-500 tracking-wider font-medium mb-4", "System Health" }
                    div { class: "flex items-center justify-around",
                        div { class: "flex flex-col items-center",
                            CpuGauge { percentage: health.cpu_percent as f64, size: 64 }
                            div { class: "text-[10px] mt-1.5 text-gray-400", "CPU" }
                        }
                        div { class: "flex flex-col items-center w-24",
                            MemoryProgress { used_gb: (health.memory_percent / 100.0 * 16.0) as f64, total_gb: 16.0 }
                            div { class: "text-[10px] mt-1.5 text-gray-400", "Memory" }
                        }
                    }
                }

                // Card 4: Global Traffic
                div { class: "bg-glass-bg backdrop-blur-xl border border-glass-border rounded-2xl p-5 hover:border-green-500/20 transition-all duration-300",
                    div { class: "text-xs uppercase text-gray-500 tracking-wider font-medium mb-4", "Global Traffic" }
                    div { class: "grid grid-cols-2 gap-3",
                        div {
                            div { class: "text-xl font-bold text-blue-400", "↑ {format_bytes(total_up)}" }
                            div { class: "text-[10px] text-gray-500", "Total Upload" }
                        }
                        div {
                            div { class: "text-xl font-bold text-green-400", "↓ {format_bytes(total_down)}" }
                            div { class: "text-[10px] text-gray-500", "Total Download" }
                        }
                    }
                }
            }

            // ── Row 2: Traffic Chart + Top Connections ──────────────────
            div { class: "grid grid-cols-1 lg:grid-cols-3 gap-4",

                // Large sparkline card (2 cols)
                div { class: "lg:col-span-2 bg-glass-bg backdrop-blur-xl border border-glass-border rounded-2xl p-5 hover:border-cyan-500/10 transition-all duration-300",
                    div { class: "flex items-center justify-between mb-3",
                        h3 { class: "text-sm font-bold text-white uppercase tracking-wider", "Traffic Throughput" }
                        span { class: "text-xs text-gray-500", "Last 60 samples" }
                    }
                    div { class: "h-32",
                        Sparkline {
                            data: global_history.clone(),
                            width: 600,
                            height: 120,
                            show_upload: true,
                            show_download: true,
                            upload_color: Some("rgba(59, 130, 246, 0.4)".to_string()),
                            download_color: Some("rgba(34, 197, 94, 0.4)".to_string()),
                        }
                    }
                }

                // Top 5 connections card
                div { class: "bg-glass-bg backdrop-blur-xl border border-glass-border rounded-2xl p-5 hover:border-purple-500/10 transition-all duration-300",
                    h3 { class: "text-sm font-bold text-white uppercase tracking-wider mb-3", "Top Connections" }
                    div { class: "space-y-2.5",
                        for (i, conn) in top_conns.iter().enumerate() {
                            div { class: "flex items-center justify-between py-1.5 border-b border-white/5 last:border-0",
                                div { class: "flex items-center gap-2 min-w-0",
                                    span { class: "text-[10px] text-gray-600 w-4", "#{i + 1}" }
                                    div { class: "min-w-0",
                                        div { class: "text-xs text-white truncate font-mono", "{conn.remote_ip}" }
                                        div { class: "text-[10px] text-gray-500", "{conn.protocol} · {conn.transport}" }
                                    }
                                }
                                div { class: "text-right text-[10px] font-mono shrink-0",
                                    div { class: "text-blue-400", "↑{format_bytes(conn.upload_bytes)}" }
                                    div { class: "text-green-400", "↓{format_bytes(conn.download_bytes)}" }
                                }
                            }
                        }
                        if top_conns.is_empty() {
                            div { class: "text-center text-gray-600 text-xs py-4", "No active connections" }
                        }
                    }
                }
            }

            // ── Row 3: Full-width Active Connections Table ──────────────
            div { class: "bg-glass-bg backdrop-blur-xl border border-glass-border rounded-2xl overflow-hidden hover:border-white/10 transition-all duration-300",
                div { class: "px-5 py-4 border-b border-white/[0.06] flex justify-between items-center",
                    h3 { class: "text-sm font-bold text-white uppercase tracking-wider", "Active Connections" }
                    span { class: "text-xs text-gray-500 bg-white/5 px-2.5 py-1 rounded-lg", "{s.active_connections.len()} sessions" }
                }

                div { class: "overflow-x-auto",
                    table { class: "w-full text-left border-collapse",
                        thead { class: "bg-black/20 text-[10px] text-gray-500 uppercase tracking-wider",
                            tr {
                                th { class: "px-4 py-2.5 font-medium", "ID" }
                                th { class: "px-4 py-2.5 font-medium", "Local" }
                                th { class: "px-4 py-2.5 font-medium", "Remote" }
                                th { class: "px-4 py-2.5 font-medium", "Proto" }
                                th { class: "px-4 py-2.5 font-medium", "Transport" }
                                th { class: "px-4 py-2.5 font-medium text-right", "RTT" }
                                th { class: "px-4 py-2.5 font-medium text-right", "Jitter" }
                                th { class: "px-4 py-2.5 font-medium w-32 text-center", "Activity" }
                                th { class: "px-4 py-2.5 font-medium text-right", "Traffic" }
                            }
                        }
                        tbody { class: "text-xs divide-y divide-white/5 font-mono",
                            for conn in &s.active_connections {
                                tr { class: "hover:bg-white/[0.03] transition-colors duration-150",
                                    td { class: "px-4 py-2.5 text-gray-500", "{conn.id}" }
                                    td { class: "px-4 py-2.5 text-blue-300", "{conn.local_ip}" }
                                    td { class: "px-4 py-2.5 text-purple-300", "{conn.remote_ip}" }
                                    td { class: "px-4 py-2.5",
                                        span { class: "px-1.5 py-0.5 rounded bg-white/5 text-gray-300 text-[10px]", "{conn.protocol}" }
                                    }
                                    td { class: "px-4 py-2.5 text-gray-300", "{conn.transport}" }
                                    td { class: "px-4 py-2.5 text-right",
                                        span { class: "{rtt_color(conn.rtt_ms)}", "{conn.rtt_ms:.0}ms" }
                                    }
                                    td { class: "px-4 py-2.5 text-right text-gray-500", "{conn.jitter_ms:.1}ms" }
                                    td { class: "px-4 py-1",
                                        if let Some(hist) = history_map.get(&conn.id) {
                                            Sparkline {
                                                data: hist.clone(),
                                                width: 100,
                                                height: 24,
                                                show_upload: true,
                                                show_download: true,
                                                upload_color: Some("rgba(59, 130, 246, 0.3)".to_string()),
                                                download_color: Some("rgba(34, 197, 94, 0.3)".to_string()),
                                            }
                                        }
                                    }
                                    td { class: "px-4 py-2.5 text-right",
                                        div { class: "flex flex-col items-end gap-0.5",
                                            span { class: "text-blue-400", "↑ {format_bytes(conn.upload_bytes)}" }
                                            span { class: "text-green-400", "↓ {format_bytes(conn.download_bytes)}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
