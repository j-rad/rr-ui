// src/ui/components/traffic_matrix.rs
//! WebGL Traffic Matrix Component
//!
//! Real-time 3D visualization of network traffic using canvas/WebGL

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

/// Traffic node for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficNode {
    pub id: String,
    pub name: String,
    pub health: f32, // 0.0 - 1.0
    pub connections: usize,
    pub bandwidth: f64, // bytes/sec
}

/// Connection between nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficConnection {
    pub from: String,
    pub to: String,
    pub bandwidth: f64,
    pub latency_ms: f32,
}

/// Traffic matrix data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrafficMatrixData {
    pub nodes: Vec<TrafficNode>,
    pub connections: Vec<TrafficConnection>,
    pub total_bandwidth: f64,
    pub timestamp: u64,
}

/// WebGL Traffic Matrix Component
#[component]
pub fn TrafficMatrix() -> Element {
    let mut canvas_initialized = use_signal(|| false);
    let mut is_loading = use_signal(|| true);
    let mut matrix_data = use_signal(TrafficMatrixData::default);
    let mut selected_node = use_signal(|| None::<String>);

    // Initialize canvas on mount
    use_effect(move || {
        if !canvas_initialized() {
            spawn(async move {
                #[cfg(feature = "web")]
                {
                    gloo_timers::future::TimeoutFuture::new(100).await;
                }
                canvas_initialized.set(true);
                is_loading.set(false);
            });
        }
    });

    // Simulate real-time data updates
    use_effect(move || {
        spawn(async move {
            loop {
                #[cfg(feature = "web")]
                {
                    gloo_timers::future::TimeoutFuture::new(2000).await;
                }

                #[cfg(not(feature = "web"))]
                {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }

                // Simulate traffic data update
                let nodes = vec![
                    TrafficNode {
                        id: "node-1".into(),
                        name: "US Edge".into(),
                        health: 0.95,
                        connections: 150,
                        bandwidth: 1_500_000.0,
                    },
                    TrafficNode {
                        id: "node-2".into(),
                        name: "EU Gateway".into(),
                        health: 0.87,
                        connections: 230,
                        bandwidth: 2_300_000.0,
                    },
                    TrafficNode {
                        id: "node-3".into(),
                        name: "Asia Hub".into(),
                        health: 0.92,
                        connections: 180,
                        bandwidth: 1_800_000.0,
                    },
                    TrafficNode {
                        id: "node-4".into(),
                        name: "Origin".into(),
                        health: 1.0,
                        connections: 560,
                        bandwidth: 5_600_000.0,
                    },
                ];

                let connections = vec![
                    TrafficConnection {
                        from: "node-1".into(),
                        to: "node-4".into(),
                        bandwidth: 1_500_000.0,
                        latency_ms: 45.0,
                    },
                    TrafficConnection {
                        from: "node-2".into(),
                        to: "node-4".into(),
                        bandwidth: 2_300_000.0,
                        latency_ms: 32.0,
                    },
                    TrafficConnection {
                        from: "node-3".into(),
                        to: "node-4".into(),
                        bandwidth: 1_800_000.0,
                        latency_ms: 78.0,
                    },
                ];

                matrix_data.set(TrafficMatrixData {
                    nodes,
                    connections,
                    total_bandwidth: 5_600_000.0,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                });
            }
        });
    });

    let format_bandwidth = |bps: f64| -> String {
        if bps >= 1_000_000_000.0 {
            format!("{:.1} Gbps", bps / 1_000_000_000.0)
        } else if bps >= 1_000_000.0 {
            format!("{:.1} Mbps", bps / 1_000_000.0)
        } else if bps >= 1_000.0 {
            format!("{:.1} Kbps", bps / 1_000.0)
        } else {
            format!("{:.0} bps", bps)
        }
    };

    let get_health_color = |health: f32| -> &'static str {
        if health >= 0.9 {
            "#22C55E" // Green
        } else if health >= 0.7 {
            "#F59E0B" // Amber
        } else {
            "#EF4444" // Red
        }
    };

    rsx! {
        div { class: "traffic-matrix-container card",
            // Header
            div { class: "traffic-matrix-header",
                h3 { "Network Traffic Matrix" }
                div { class: "traffic-stats",
                    span { class: "stat",
                        i { class: "fas fa-server" }
                        " {matrix_data().nodes.len()} nodes"
                    }
                    span { class: "stat",
                        i { class: "fas fa-exchange-alt" }
                        " {format_bandwidth(matrix_data().total_bandwidth * 8.0)}"
                    }
                }
            }

            // Canvas area
            div { class: "traffic-matrix-canvas",
                if is_loading() {
                    div { class: "loading-overlay",
                        div { class: "loading-shimmer" }
                        span { "Initializing visualization..." }
                    }
                } else {
                    // SVG-based visualization (fallback for WebGL)
                    {
                        let data = matrix_data();
                        let connections = data.connections.clone();
                        let nodes = data.nodes.clone();

                        rsx! {
                            svg {
                                class: "traffic-svg",
                                view_box: "0 0 400 300",

                                // Draw connections first (under nodes)
                                for conn in connections.iter() {
                                    line {
                                        class: "traffic-connection",
                                        x1: "200",
                                        y1: "150",
                                        x2: match conn.from.as_str() {
                                            "node-1" => "100",
                                            "node-2" => "300",
                                            "node-3" => "200",
                                            _ => "200"
                                        },
                                        y2: match conn.from.as_str() {
                                            "node-1" => "80",
                                            "node-2" => "80",
                                            "node-3" => "250",
                                            _ => "150"
                                        },
                                        stroke: "#06B6D4",
                                        stroke_width: "{(conn.bandwidth / 1_000_000.0).max(1.0).min(5.0)}",
                                        stroke_opacity: "0.6"
                                    }
                                }

                                // Draw nodes as simple circles with fixed positions
                                for (i, node) in nodes.iter().enumerate() {
                                    {
                                        let (cx, cy) = match i {
                                            0 => (100, 80),
                                            1 => (300, 80),
                                            2 => (200, 250),
                                            3 => (200, 150),
                                            _ => (200, 150),
                                        };
                                        let is_selected = selected_node().as_ref() == Some(&node.id);
                                        let radius = if is_selected { 30 } else { 25 };
                                        let health_color = get_health_color(node.health);
                                        let node_id = node.id.clone();
                                        let node_name = node.name.clone();

                                        rsx! {
                                            g {
                                                class: "traffic-node",
                                                onclick: move |_| {
                                                    selected_node.set(Some(node_id.clone()));
                                                },

                                                // Main node circle
                                                circle {
                                                    cx: "{cx}",
                                                    cy: "{cy}",
                                                    r: "{radius}",
                                                    fill: "#1E293B",
                                                    stroke: "{health_color}",
                                                    stroke_width: if is_selected { "3" } else { "2" }
                                                }

                                                // Node label
                                                text {
                                                    x: "{cx}",
                                                    y: "{cy + 45}",
                                                    text_anchor: "middle",
                                                    fill: "#94A3B8",
                                                    font_size: "10",
                                                    "{node_name}"
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

            // Node details panel
            if let Some(node_id) = selected_node() {
                if let Some(node) = matrix_data().nodes.iter().find(|n| n.id == node_id) {
                    div { class: "node-details-panel",
                        h4 { "{node.name}" }
                        div { class: "node-stats",
                            div { class: "stat-row",
                                span { class: "label", "Health" }
                                span {
                                    class: "value",
                                    style: "color: {get_health_color(node.health)}",
                                    "{(node.health * 100.0) as u32}%"
                                }
                            }
                            div { class: "stat-row",
                                span { class: "label", "Connections" }
                                span { class: "value", "{node.connections}" }
                            }
                            div { class: "stat-row",
                                span { class: "label", "Bandwidth" }
                                span { class: "value", "{format_bandwidth(node.bandwidth * 8.0)}" }
                            }
                        }
                        button {
                            class: "btn btn-sm btn-secondary",
                            onclick: move |_| selected_node.set(None),
                            "Close"
                        }
                    }
                }
            }
        }
    }
}
