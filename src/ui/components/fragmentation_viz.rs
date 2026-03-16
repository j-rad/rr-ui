//! Fragmentation Visualization
//!
//! Visualizes packet fragmentation when **Flow-J** or **Fragment** protocol
//! is active. The main routing link forks into parallel sub-lines, each
//! carrying its own animated dot, illustrating how traffic is split across
//! multiple fragment streams.

use crate::ui::theme;
use dioxus::prelude::*;

// ─── Data Model ────────────────────────────────────────────────────────────────

/// Protocol responsible for the fragmentation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FragmentProtocol {
    /// Flow-J fragmentation protocol.
    FlowJ,
    /// Generic Fragment protocol.
    Fragment,
}

impl FragmentProtocol {
    /// Display label.
    pub fn label(&self) -> &'static str {
        match self {
            FragmentProtocol::FlowJ => "FLOW-J",
            FragmentProtocol::Fragment => "FRAGMENT",
        }
    }

    /// Accent color for the fragmentation overlay.
    pub fn color(&self) -> &'static str {
        match self {
            FragmentProtocol::FlowJ => theme::COLOR_CYBER_PURPLE,
            FragmentProtocol::Fragment => theme::COLOR_ELECTRIC_CYAN,
        }
    }
}

/// A single fragment stream within a fragmented connection.
#[derive(Clone, Debug, PartialEq)]
pub struct FragmentStream {
    /// Unique stream identifier.
    pub stream_id: String,
    /// Number of fragments this stream is split into.
    pub fragment_count: u32,
    /// Protocol driving the fragmentation.
    pub protocol: FragmentProtocol,
    /// Average bytes per fragment.
    pub bytes_per_fragment: u32,
}

/// The current fragmentation state of the connection.
#[derive(Clone, Debug, PartialEq)]
pub enum FragmentationState {
    /// No fragmentation active — single-stream transmission.
    Inactive,
    /// Fragmentation is active with one or more parallel streams.
    Active {
        /// The active fragment streams.
        streams: Vec<FragmentStream>,
    },
}

impl FragmentationState {
    /// Returns the number of active streams (0 if inactive).
    pub fn stream_count(&self) -> usize {
        match self {
            FragmentationState::Inactive => 0,
            FragmentationState::Active { streams } => streams.len(),
        }
    }

    /// Whether fragmentation is currently active.
    pub fn is_active(&self) -> bool {
        matches!(self, FragmentationState::Active { .. })
    }
}

// ─── Layout Engine ─────────────────────────────────────────────────────────────

/// Computes the Y-positions for forked sub-lines originating at `origin_y`.
///
/// The streams are spread evenly across a `spread_height` centered on `origin_y`.
///
/// Returns a vector of `(fork_y, stream_index)` pairs.
///
/// # Edge cases
/// - 0 streams → empty result
/// - 1 stream → returns `origin_y` (no fork needed)
pub fn compute_fork_paths(
    origin_y: f32,
    stream_count: usize,
    spread_height: f32,
) -> Vec<(f32, usize)> {
    if stream_count == 0 {
        return Vec::new();
    }
    if stream_count == 1 {
        return vec![(origin_y, 0)];
    }

    let half = spread_height / 2.0;
    let step = spread_height / (stream_count - 1) as f32;
    (0..stream_count)
        .map(|i| {
            let y = origin_y - half + (i as f32 * step);
            (y, i)
        })
        .collect()
}

// ─── Component Props ───────────────────────────────────────────────────────────

/// Props for the FragmentationViz component.
#[derive(Props, Clone, PartialEq)]
pub struct FragmentationVizProps {
    /// Current fragmentation state.
    pub state: FragmentationState,

    /// X position where the fork originates (e.g. midpoint of a routing link).
    #[props(default = 100.0)]
    pub fork_x: f32,

    /// X position where the sub-lines terminate.
    #[props(default = 300.0)]
    pub end_x: f32,

    /// Y center of the fork point.
    #[props(default = 160.0)]
    pub origin_y: f32,

    /// Total height available for spreading sub-lines.
    #[props(default = 120.0)]
    pub spread_height: f32,

    /// SVG viewport width.
    #[props(default = 400)]
    pub width: u32,

    /// SVG viewport height.
    #[props(default = 320)]
    pub height: u32,
}

// ─── Component ─────────────────────────────────────────────────────────────────

/// Visualizes packet fragmentation as forking sub-lines with animated dots.
///
/// When `FragmentationState::Inactive`, renders a single pass-through line.
/// When `FragmentationState::Active`, the line forks at `fork_x` into
/// `stream_count` parallel sub-lines, each with its own travelling dot.
#[component]
pub fn FragmentationViz(props: FragmentationVizProps) -> Element {
    let w = props.width as f32;
    let h = props.height as f32;
    let dot_r = theme::ROUTING_DOT_RADIUS;

    let fork_anim = r#"
        @keyframes edgeray-frag-dot {
            0%   { offset-distance: 0%; opacity: 0.9; }
            100% { offset-distance: 100%; opacity: 0.9; }
        }
        @keyframes edgeray-split-marker {
            0%, 100% { r: 5; opacity: 0.6; }
            50% { r: 8; opacity: 1.0; }
        }
    "#;

    rsx! {
        div { class: "relative",
            style { {fork_anim} }

            svg {
                class: "block",
                width: "{w}",
                height: "{h}",
                view_box: "0 0 {w} {h}",

                match &props.state {
                    FragmentationState::Inactive => rsx! {
                        // Single pass-through line
                        line {
                            x1: "{props.fork_x}",
                            y1: "{props.origin_y}",
                            x2: "{props.end_x}",
                            y2: "{props.origin_y}",
                            stroke: "{theme::ROUTING_LINK_COLOR}",
                            stroke_width: "1.5",
                            stroke_dasharray: "6 4",
                        }
                    },
                    FragmentationState::Active { streams } => {
                        let fork_paths = compute_fork_paths(
                            props.origin_y,
                            streams.len(),
                            props.spread_height,
                        );

                        rsx! {
                            // Split-point marker
                            circle {
                                cx: "{props.fork_x}",
                                cy: "{props.origin_y}",
                                r: "5",
                                fill: "{theme::COLOR_CYBER_PURPLE}",
                                style: "animation: edgeray-split-marker 1.5s ease-in-out infinite;",
                            }

                            // Protocol label
                            {
                                let proto_label = streams.first()
                                    .map(|s| s.protocol.label())
                                    .unwrap_or("SPLIT");
                                let proto_color = streams.first()
                                    .map(|s| s.protocol.color())
                                    .unwrap_or(theme::COLOR_CYBER_PURPLE);

                                rsx! {
                                    text {
                                        x: "{props.fork_x}",
                                        y: "{props.origin_y - 16.0}",
                                        text_anchor: "middle",
                                        fill: "{proto_color}",
                                        font_size: "9",
                                        font_weight: "700",
                                        font_family: "'JetBrains Mono', monospace",
                                        letter_spacing: "0.1em",
                                        "{proto_label}"
                                    }
                                }
                            }

                            // Forked sub-lines + dots
                            for (fork_y , idx) in &fork_paths {
                                {
                                    let stream = &streams[*idx];
                                    let color = stream.protocol.color();
                                    let dur = 2.0 - (stream.bytes_per_fragment as f32 / 2000.0).clamp(0.0, 1.5);
                                    let path_d = format!(
                                        "M {},{} C {},{} {},{} {},{}",
                                        props.fork_x, props.origin_y,
                                        props.fork_x + 40.0, props.origin_y,
                                        props.fork_x + 40.0, fork_y,
                                        props.end_x, fork_y,
                                    );
                                    let line_key = format!("frag-line-{}", idx);
                                    let dot_key = format!("frag-dot-{}", idx);
                                    let frag_label = format!("×{}", stream.fragment_count);
                                    let label_key = format!("frag-lbl-{}", idx);

                                    rsx! {
                                        // Sub-line (cubic bezier)
                                        path {
                                            key: "{line_key}",
                                            d: "{path_d}",
                                            fill: "none",
                                            stroke: "{color}",
                                            stroke_width: "1",
                                            stroke_dasharray: "4 3",
                                            opacity: "0.5",
                                        }

                                        // Travelling dot
                                        circle {
                                            key: "{dot_key}",
                                            r: "{dot_r}",
                                            fill: "{color}",
                                            style: "offset-path: path('{path_d}'); animation: edgeray-frag-dot {dur}s linear infinite;",
                                        }

                                        // Fragment count label
                                        text {
                                            key: "{label_key}",
                                            x: "{props.end_x + 8.0}",
                                            y: "{fork_y + 3.0}",
                                            fill: "rgba(255,255,255,0.40)",
                                            font_size: "9",
                                            font_family: "'JetBrains Mono', monospace",
                                            "{frag_label}"
                                        }
                                    }
                                }
                            }
                        }
                    },
                }
            }
        }
    }
}

// ─── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fragment_protocol_labels() {
        assert_eq!(FragmentProtocol::FlowJ.label(), "FLOW-J");
        assert_eq!(FragmentProtocol::Fragment.label(), "FRAGMENT");
    }

    #[test]
    fn test_fragmentation_state_stream_count() {
        assert_eq!(FragmentationState::Inactive.stream_count(), 0);
        assert!(!FragmentationState::Inactive.is_active());

        let active = FragmentationState::Active {
            streams: vec![
                FragmentStream {
                    stream_id: "s1".into(),
                    fragment_count: 4,
                    protocol: FragmentProtocol::FlowJ,
                    bytes_per_fragment: 512,
                },
                FragmentStream {
                    stream_id: "s2".into(),
                    fragment_count: 8,
                    protocol: FragmentProtocol::FlowJ,
                    bytes_per_fragment: 256,
                },
            ],
        };
        assert_eq!(active.stream_count(), 2);
        assert!(active.is_active());
    }

    #[test]
    fn test_compute_fork_paths_zero_streams() {
        let paths = compute_fork_paths(100.0, 0, 120.0);
        assert!(paths.is_empty());
    }

    #[test]
    fn test_compute_fork_paths_single_stream() {
        let paths = compute_fork_paths(100.0, 1, 120.0);
        assert_eq!(paths.len(), 1);
        assert!((paths[0].0 - 100.0).abs() < f32::EPSILON);
        assert_eq!(paths[0].1, 0);
    }

    #[test]
    fn test_compute_fork_paths_two_streams() {
        let paths = compute_fork_paths(100.0, 2, 80.0);
        assert_eq!(paths.len(), 2);
        // First stream: 100 - 40 = 60
        assert!((paths[0].0 - 60.0).abs() < f32::EPSILON);
        // Second stream: 100 + 40 = 140
        assert!((paths[1].0 - 140.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_compute_fork_paths_even_distribution() {
        let paths = compute_fork_paths(200.0, 5, 200.0);
        assert_eq!(paths.len(), 5);

        // Should be evenly spaced across 200px centered on 200
        // Range: 100 to 300, step = 50
        let expected_ys = [100.0, 150.0, 200.0, 250.0, 300.0];
        for (i, (y, idx)) in paths.iter().enumerate() {
            assert!(
                (y - expected_ys[i]).abs() < 0.1,
                "Path {} y={} expected {}",
                i,
                y,
                expected_ys[i]
            );
            assert_eq!(*idx, i);
        }
    }

    #[test]
    fn test_compute_fork_paths_symmetry() {
        let origin = 150.0;
        let spread = 100.0;
        let paths = compute_fork_paths(origin, 3, spread);
        // Three streams: origin-50, origin, origin+50
        assert!((paths[0].0 - 100.0).abs() < 0.1);
        assert!((paths[1].0 - 150.0).abs() < 0.1);
        assert!((paths[2].0 - 200.0).abs() < 0.1);
    }
}
