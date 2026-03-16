//! Routing Logic Canvas
//!
//! A simplified node-link diagram showing traffic flow from
//! **App Source** → **Logical Filter** → **Outbound Node**.
//!
//! Small animated "packet" dots travel along the links at speeds
//! proportional to the traffic volume on each link.

use crate::ui::components::glass_card::GlassCard;
use crate::ui::theme;
use dioxus::prelude::*;

// ─── Node / Link Models ────────────────────────────────────────────────────────

/// The logical type of a routing node.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NodeType {
    /// The originating application or device.
    AppSource,
    /// A logical filter / rule evaluation point.
    LogicalFilter,
    /// An outbound proxy node (VLESS, Trojan, etc.).
    OutboundNode,
}

impl NodeType {
    /// Column index in the left-to-right layout (0-based).
    pub fn column(&self) -> u8 {
        match self {
            NodeType::AppSource => 0,
            NodeType::LogicalFilter => 1,
            NodeType::OutboundNode => 2,
        }
    }

    /// Accent color for the node.
    pub fn color(&self) -> &'static str {
        match self {
            NodeType::AppSource => theme::COLOR_CYBER_PURPLE,
            NodeType::LogicalFilter => theme::COLOR_ELECTRIC_CYAN,
            NodeType::OutboundNode => theme::COLOR_EMERALD,
        }
    }

    /// Label prefix displayed above the node icon.
    pub fn tier_label(&self) -> &'static str {
        match self {
            NodeType::AppSource => "SOURCE",
            NodeType::LogicalFilter => "FILTER",
            NodeType::OutboundNode => "OUTBOUND",
        }
    }
}

/// A node in the routing canvas.
#[derive(Clone, Debug, PartialEq)]
pub struct RoutingNode {
    /// Unique identifier (e.g. "app-0", "filter-geo", "outbound-reality-1").
    pub id: String,
    /// Human-readable label displayed beneath the node.
    pub label: String,
    /// Logical type determining column placement.
    pub node_type: NodeType,
    /// Computed X position in pixels (set by `compute_positions`).
    pub x: f32,
    /// Computed Y position in pixels (set by `compute_positions`).
    pub y: f32,
}

/// A directed link between two routing nodes.
#[derive(Clone, Debug, PartialEq)]
pub struct RoutingLink {
    /// Source node ID.
    pub source_id: String,
    /// Target node ID.
    pub target_id: String,
    /// Traffic volume in bytes/sec — controls dot animation speed.
    pub traffic_volume: u64,
}

impl RoutingLink {
    /// Animation duration in seconds for the travelling dot.
    /// Higher traffic → faster animation (shorter duration).
    pub fn dot_duration_secs(&self) -> f32 {
        // Range: 0.5s (very fast) .. 4.0s (idle trickle)
        let vol = self.traffic_volume as f32;
        if vol <= 0.0 {
            return 4.0;
        }
        let t = (vol / 10_000_000.0).clamp(0.0, 1.0); // 10 MB/s → max speed
        4.0 - t * 3.5
    }
}

// ─── Layout Engine ─────────────────────────────────────────────────────────────

/// Computes x/y positions for each node in a three-column left-to-right layout.
///
/// Nodes are grouped by `NodeType::column()` and distributed vertically
/// within their column with even spacing.
///
/// This is a **pure function** with no side effects — fully testable.
pub fn compute_positions(nodes: &mut [RoutingNode], canvas_width: f32, canvas_height: f32) {
    if nodes.is_empty() || canvas_width <= 0.0 || canvas_height <= 0.0 {
        return;
    }

    // Horizontal positions for the 3 columns
    let col_x = [
        canvas_width * 0.15,
        canvas_width * 0.50,
        canvas_width * 0.85,
    ];

    // Count nodes per column to distribute vertically
    let mut col_counts = [0u32; 3];
    for node in nodes.iter() {
        let c = node.node_type.column() as usize;
        if c < 3 {
            col_counts[c] += 1;
        }
    }

    // Track current index within each column
    let mut col_indices = [0u32; 3];

    for node in nodes.iter_mut() {
        let c = node.node_type.column() as usize;
        if c >= 3 {
            continue;
        }
        node.x = col_x[c];

        let count = col_counts[c];
        let index = col_indices[c];
        if count == 1 {
            node.y = canvas_height / 2.0;
        } else {
            let spacing = canvas_height / (count + 1) as f32;
            node.y = spacing * (index + 1) as f32;
        }
        col_indices[c] += 1;
    }
}

/// Looks up the canvas position of an outbound node by its ID.
///
/// Returns `Some((x, y))` if the node exists and has been positioned.
pub fn outbound_position(outbound_id: &str, nodes: &[RoutingNode]) -> Option<(f32, f32)> {
    nodes
        .iter()
        .find(|n| n.id == outbound_id && n.node_type == NodeType::OutboundNode)
        .map(|n| (n.x, n.y))
}

// ─── Component Props ───────────────────────────────────────────────────────────

/// Props for the RoutingLogicCanvas component.
#[derive(Props, Clone, PartialEq)]
pub struct RoutingLogicCanvasProps {
    /// The routing nodes to display.
    pub nodes: Vec<RoutingNode>,

    /// The links connecting nodes.
    pub links: Vec<RoutingLink>,

    /// Canvas width in pixels.
    #[props(default = 600)]
    pub width: u32,

    /// Canvas height in pixels.
    #[props(default = 320)]
    pub height: u32,
}

// ─── Component ─────────────────────────────────────────────────────────────────

/// A visual routing map showing traffic flow through EdgeRay's routing engine.
///
/// Renders:
/// - **Nodes** as labeled circles colour-coded by tier
/// - **Links** as dashed lines connecting source → filter → outbound
/// - **Packet dots** — small glowing circles that travel along links,
///   animated at speeds proportional to traffic volume
#[component]
pub fn RoutingLogicCanvas(props: RoutingLogicCanvasProps) -> Element {
    let w = props.width as f32;
    let h = props.height as f32;
    let r = theme::ROUTING_NODE_RADIUS;
    let dot_r = theme::ROUTING_DOT_RADIUS;

    // Compute positions (clone to avoid mutating props)
    let mut positioned_nodes = props.nodes.clone();
    compute_positions(&mut positioned_nodes, w, h);

    // Build an animation keyframe for the travelling dot
    let dot_keyframes = r#"
        @keyframes edgeray-packet-travel {
            0%   { offset-distance: 0%; opacity: 0.9; }
            100% { offset-distance: 100%; opacity: 0.9; }
        }
        @keyframes edgeray-packet-glow {
            0%, 100% { filter: drop-shadow(0 0 2px rgba(0,242,255,0.6)); }
            50% { filter: drop-shadow(0 0 6px rgba(0,242,255,0.9)); }
        }
    "#;

    rsx! {
        GlassCard {
            title: "ROUTING MAP".to_string(),
            class: "min-h-[200px]".to_string(),

            style { {dot_keyframes} }

            svg {
                class: "block",
                width: "{w}",
                height: "{h}",
                view_box: "0 0 {w} {h}",

                // ── Links ──
                for link in &props.links {
                    {
                        let src = positioned_nodes.iter().find(|n| n.id == link.source_id);
                        let tgt = positioned_nodes.iter().find(|n| n.id == link.target_id);
                        if let (Some(s), Some(t)) = (src, tgt) {
                            let dur = link.dot_duration_secs();
                            let sx = s.x;
                            let sy = s.y;
                            let tx = t.x;
                            let ty = t.y;
                            let link_id = format!("link-{}-{}", link.source_id, link.target_id);
                            let path_d = format!("M {},{} L {},{}", sx, sy, tx, ty);

                            rsx! {
                                // Link line
                                line {
                                    key: "{link_id}",
                                    x1: "{sx}",
                                    y1: "{sy}",
                                    x2: "{tx}",
                                    y2: "{ty}",
                                    stroke: "{theme::ROUTING_LINK_COLOR}",
                                    stroke_width: "1.5",
                                    stroke_dasharray: "6 4",
                                    stroke_linecap: "round",
                                }

                                // Animated packet dot
                                circle {
                                    key: "dot-{link_id}",
                                    r: "{dot_r}",
                                    fill: "{theme::COLOR_ELECTRIC_CYAN}",
                                    style: "offset-path: path('{path_d}'); animation: edgeray-packet-travel {dur}s linear infinite, edgeray-packet-glow 1s ease-in-out infinite;",
                                }
                            }
                        } else {
                            rsx! {}
                        }
                    }
                }

                // ── Nodes ──
                for node in &positioned_nodes {
                    {
                        let color = node.node_type.color();
                        let tier = node.node_type.tier_label();
                        let nx = node.x;
                        let ny = node.y;
                        let node_key = node.id.clone();
                        let node_label = node.label.clone();

                        rsx! {
                            // Ambient glow
                            circle {
                                key: "glow-{node_key}",
                                cx: "{nx}",
                                cy: "{ny}",
                                r: "{r + 8.0}",
                                fill: "{color}",
                                opacity: "0.08",
                            }

                            // Node body
                            circle {
                                key: "node-{node_key}",
                                cx: "{nx}",
                                cy: "{ny}",
                                r: "{r}",
                                fill: "{theme::COLOR_VOID}",
                                stroke: "{color}",
                                stroke_width: "2",
                            }

                            // Tier label (above node)
                            text {
                                key: "tier-{node_key}",
                                x: "{nx}",
                                y: "{ny - r - 14.0}",
                                text_anchor: "middle",
                                fill: "rgba(255,255,255,0.35)",
                                font_size: "8",
                                font_family: "'JetBrains Mono', monospace",
                                letter_spacing: "0.15em",
                                "{tier}"
                            }

                            // Node label (below node)
                            text {
                                key: "label-{node_key}",
                                x: "{nx}",
                                y: "{ny + r + 16.0}",
                                text_anchor: "middle",
                                fill: "{color}",
                                font_size: "10",
                                font_weight: "600",
                                font_family: "Inter, sans-serif",
                                "{node_label}"
                            }
                        }
                    }
                }
            }
        }
    }
}

// ─── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_node(id: &str, label: &str, node_type: NodeType) -> RoutingNode {
        RoutingNode {
            id: id.to_string(),
            label: label.to_string(),
            node_type,
            x: 0.0,
            y: 0.0,
        }
    }

    #[test]
    fn test_node_type_columns() {
        assert_eq!(NodeType::AppSource.column(), 0);
        assert_eq!(NodeType::LogicalFilter.column(), 1);
        assert_eq!(NodeType::OutboundNode.column(), 2);
    }

    #[test]
    fn test_node_type_colors() {
        assert_eq!(NodeType::AppSource.color(), theme::COLOR_CYBER_PURPLE);
        assert_eq!(NodeType::LogicalFilter.color(), theme::COLOR_ELECTRIC_CYAN);
        assert_eq!(NodeType::OutboundNode.color(), theme::COLOR_EMERALD);
    }

    #[test]
    fn test_compute_positions_three_tier() {
        let mut nodes = vec![
            make_node("app-0", "App", NodeType::AppSource),
            make_node("filter-0", "GeoIP", NodeType::LogicalFilter),
            make_node("outbound-1", "REALITY-1", NodeType::OutboundNode),
        ];

        compute_positions(&mut nodes, 600.0, 300.0);

        // Column 0: x = 600 * 0.15 = 90
        assert!((nodes[0].x - 90.0).abs() < 0.1);
        // Column 1: x = 600 * 0.50 = 300
        assert!((nodes[1].x - 300.0).abs() < 0.1);
        // Column 2: x = 600 * 0.85 = 510
        assert!((nodes[2].x - 510.0).abs() < 0.1);

        // Single node in each column → centered at height/2
        assert!((nodes[0].y - 150.0).abs() < 0.1);
        assert!((nodes[1].y - 150.0).abs() < 0.1);
        assert!((nodes[2].y - 150.0).abs() < 0.1);
    }

    #[test]
    fn test_compute_positions_multiple_outbounds() {
        let mut nodes = vec![
            make_node("app-0", "App", NodeType::AppSource),
            make_node("outbound-1", "REALITY-1", NodeType::OutboundNode),
            make_node("outbound-2", "REALITY-2", NodeType::OutboundNode),
            make_node("outbound-3", "VMess", NodeType::OutboundNode),
        ];

        compute_positions(&mut nodes, 600.0, 400.0);

        // 3 outbound nodes in column 2: spacing = 400 / 4 = 100
        // y positions: 100, 200, 300
        assert!((nodes[1].y - 100.0).abs() < 0.1);
        assert!((nodes[2].y - 200.0).abs() < 0.1);
        assert!((nodes[3].y - 300.0).abs() < 0.1);
    }

    #[test]
    fn test_compute_positions_empty() {
        let mut nodes: Vec<RoutingNode> = vec![];
        compute_positions(&mut nodes, 600.0, 300.0);
        assert!(nodes.is_empty());
    }

    #[test]
    fn test_outbound_position_found() {
        let mut nodes = vec![
            make_node("app-0", "App", NodeType::AppSource),
            make_node("outbound-1", "REALITY-1", NodeType::OutboundNode),
        ];
        compute_positions(&mut nodes, 600.0, 300.0);

        let pos = outbound_position("outbound-1", &nodes);
        assert!(pos.is_some());
        let (x, y) = pos.unwrap();
        assert!((x - 510.0).abs() < 0.1);
        assert!((y - 150.0).abs() < 0.1);
    }

    #[test]
    fn test_outbound_position_not_found() {
        let nodes = vec![make_node("app-0", "App", NodeType::AppSource)];
        assert!(outbound_position("outbound-999", &nodes).is_none());
    }

    #[test]
    fn test_outbound_position_wrong_type() {
        // Node exists but is not OutboundNode
        let mut nodes = vec![make_node("filter-0", "GeoIP", NodeType::LogicalFilter)];
        compute_positions(&mut nodes, 600.0, 300.0);
        assert!(outbound_position("filter-0", &nodes).is_none());
    }

    #[test]
    fn test_link_dot_duration() {
        // Zero traffic → slowest
        let idle = RoutingLink {
            source_id: "a".into(),
            target_id: "b".into(),
            traffic_volume: 0,
        };
        assert!((idle.dot_duration_secs() - 4.0).abs() < f32::EPSILON);

        // Max traffic → fastest
        let fast = RoutingLink {
            source_id: "a".into(),
            target_id: "b".into(),
            traffic_volume: 100_000_000,
        };
        assert!((fast.dot_duration_secs() - 0.5).abs() < f32::EPSILON);

        // Mid traffic
        let mid = RoutingLink {
            source_id: "a".into(),
            target_id: "b".into(),
            traffic_volume: 5_000_000,
        };
        assert!(mid.dot_duration_secs() > 0.5 && mid.dot_duration_secs() < 4.0);
    }
}
