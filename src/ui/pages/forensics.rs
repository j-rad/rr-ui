//! Forensics Page
//!
//! Dashboard page that composes all three Phase 3 forensics components:
//! - **ForensicsTracer** — vertical handshake timeline
//! - **RoutingLogicCanvas** — node-link routing diagram with animated packets
//! - **FragmentationViz** — packet fragmentation split-stream visualization

use crate::ui::components::forensics_tracer::*;
use crate::ui::components::fragmentation_viz::*;
use crate::ui::components::glass_card::GlassCard;
use crate::ui::components::routing_logic_canvas::*;
use dioxus::prelude::*;

// ─── Mock Data Factories ───────────────────────────────────────────────────────

/// Generates a demo trace history showing a successful VLESS/REALITY handshake.
fn demo_trace_history() -> TraceHistory {
    let mut history = TraceHistory::new(32);
    history.push(TraceEntry {
        stage: HandshakeStage::DnsResolve,
        timestamp_ms: 0,
        latency_ms: 12.0,
        status: TraceStatus::Ok,
        geo_label: Some("FRA".to_string()),
        outbound_tag: Some("reality-fra-1".to_string()),
    });
    history.push(TraceEntry {
        stage: HandshakeStage::TcpHandshake,
        timestamp_ms: 12,
        latency_ms: 45.0,
        status: TraceStatus::Ok,
        geo_label: Some("FRA".to_string()),
        outbound_tag: Some("reality-fra-1".to_string()),
    });
    history.push(TraceEntry {
        stage: HandshakeStage::TlsReality,
        timestamp_ms: 57,
        latency_ms: 78.0,
        status: TraceStatus::Ok,
        geo_label: Some("FRA".to_string()),
        outbound_tag: Some("reality-fra-1".to_string()),
    });
    history.push(TraceEntry {
        stage: HandshakeStage::Success,
        timestamp_ms: 135,
        latency_ms: 0.0,
        status: TraceStatus::Ok,
        geo_label: Some("FRA".to_string()),
        outbound_tag: Some("reality-fra-1".to_string()),
    });
    history
}

/// Generates a demo trace history showing a DPI-interfered handshake.
fn demo_dpi_trace_history() -> TraceHistory {
    let mut history = TraceHistory::new(32);
    history.push(TraceEntry {
        stage: HandshakeStage::DnsResolve,
        timestamp_ms: 0,
        latency_ms: 8.0,
        status: TraceStatus::Ok,
        geo_label: Some("SIN".to_string()),
        outbound_tag: Some("vmess-sin-2".to_string()),
    });
    history.push(TraceEntry {
        stage: HandshakeStage::TcpHandshake,
        timestamp_ms: 8,
        latency_ms: 120.0,
        status: TraceStatus::DpiDetected,
        geo_label: Some("SIN".to_string()),
        outbound_tag: Some("vmess-sin-2".to_string()),
    });
    history.push(TraceEntry {
        stage: HandshakeStage::Failed,
        timestamp_ms: 128,
        latency_ms: 0.0,
        status: TraceStatus::Timeout,
        geo_label: Some("SIN".to_string()),
        outbound_tag: Some("vmess-sin-2".to_string()),
    });
    history
}

/// Generates demo routing nodes for the canvas.
fn demo_routing_nodes() -> Vec<RoutingNode> {
    vec![
        RoutingNode {
            id: "app-0".to_string(),
            label: "EdgeRay".to_string(),
            node_type: NodeType::AppSource,
            x: 0.0,
            y: 0.0,
        },
        RoutingNode {
            id: "filter-geo".to_string(),
            label: "GeoIP Filter".to_string(),
            node_type: NodeType::LogicalFilter,
            x: 0.0,
            y: 0.0,
        },
        RoutingNode {
            id: "filter-domain".to_string(),
            label: "Domain Rules".to_string(),
            node_type: NodeType::LogicalFilter,
            x: 0.0,
            y: 0.0,
        },
        RoutingNode {
            id: "outbound-reality-1".to_string(),
            label: "REALITY FRA".to_string(),
            node_type: NodeType::OutboundNode,
            x: 0.0,
            y: 0.0,
        },
        RoutingNode {
            id: "outbound-vmess-1".to_string(),
            label: "VMess SIN".to_string(),
            node_type: NodeType::OutboundNode,
            x: 0.0,
            y: 0.0,
        },
        RoutingNode {
            id: "outbound-direct".to_string(),
            label: "DIRECT".to_string(),
            node_type: NodeType::OutboundNode,
            x: 0.0,
            y: 0.0,
        },
    ]
}

/// Generates demo routing links.
fn demo_routing_links() -> Vec<RoutingLink> {
    vec![
        RoutingLink {
            source_id: "app-0".to_string(),
            target_id: "filter-geo".to_string(),
            traffic_volume: 3_200_000,
        },
        RoutingLink {
            source_id: "app-0".to_string(),
            target_id: "filter-domain".to_string(),
            traffic_volume: 1_500_000,
        },
        RoutingLink {
            source_id: "filter-geo".to_string(),
            target_id: "outbound-reality-1".to_string(),
            traffic_volume: 2_800_000,
        },
        RoutingLink {
            source_id: "filter-domain".to_string(),
            target_id: "outbound-vmess-1".to_string(),
            traffic_volume: 800_000,
        },
        RoutingLink {
            source_id: "filter-domain".to_string(),
            target_id: "outbound-direct".to_string(),
            traffic_volume: 500_000,
        },
    ]
}

/// Generates a demo active fragmentation state.
fn demo_fragmentation() -> FragmentationState {
    FragmentationState::Active {
        streams: vec![
            FragmentStream {
                stream_id: "frag-0".to_string(),
                fragment_count: 4,
                protocol: FragmentProtocol::FlowJ,
                bytes_per_fragment: 512,
            },
            FragmentStream {
                stream_id: "frag-1".to_string(),
                fragment_count: 8,
                protocol: FragmentProtocol::FlowJ,
                bytes_per_fragment: 256,
            },
            FragmentStream {
                stream_id: "frag-2".to_string(),
                fragment_count: 6,
                protocol: FragmentProtocol::FlowJ,
                bytes_per_fragment: 384,
            },
        ],
    }
}

// ─── Page Component ────────────────────────────────────────────────────────────

/// The Forensics page composes all Phase 3 visual components into a
/// responsive dashboard layout.
#[component]
pub fn ForensicsPage() -> Element {
    let success_history = demo_trace_history();
    let dpi_history = demo_dpi_trace_history();
    let nodes = demo_routing_nodes();
    let links = demo_routing_links();
    let frag_state = demo_fragmentation();

    rsx! {
        div {
            class: "p-6 space-y-6 max-w-7xl mx-auto",

            // ── Page Header ──
            h1 {
                class: "text-2xl font-bold tracking-tight",
                style: "color: rgba(255,255,255,0.90); font-family: 'Inter', sans-serif;",
                "Visual Routing & Handshake Forensics"
            }
            p {
                class: "text-sm",
                style: "color: rgba(255,255,255,0.45); font-family: 'JetBrains Mono', monospace;",
                "Real-time handshake tracing • routing topology • packet fragmentation"
            }

            // ── Routing Canvas (full width) ──
            RoutingLogicCanvas {
                nodes: nodes,
                links: links,
                width: 800,
                height: 320,
            }

            // ── Handshake Traces (side by side) ──
            div {
                class: "grid grid-cols-1 md:grid-cols-2 gap-6",

                GlassCard {
                    title: "HANDSHAKE TRACE — REALITY FRA".to_string(),
                    ForensicsTracer {
                        history: success_history,
                        width: 360,
                        max_height: 380,
                    }
                }

                GlassCard {
                    title: "HANDSHAKE TRACE — DPI DETECTED".to_string(),
                    ForensicsTracer {
                        history: dpi_history,
                        width: 360,
                        max_height: 380,
                    }
                }
            }

            // ── Fragmentation Viz ──
            GlassCard {
                title: "PACKET FRAGMENTATION — FLOW-J".to_string(),
                FragmentationViz {
                    state: frag_state,
                    fork_x: 80.0,
                    end_x: 350.0,
                    origin_y: 160.0,
                    spread_height: 140.0,
                    width: 420,
                    height: 320,
                }
            }
        }
    }
}
