//! Handshake Forensics Tracer
//!
//! A vertical timeline component that visualizes the handshake process
//! between the client and the outbound node. Each stage (DNS Resolve,
//! TCP Handshake, TLS/REALITY, Success) is rendered as an expanding
//! circular pulse on a vertical spine.
//!
//! - **Success paths** glow Emerald (#10B981)
//! - **DPI interference** glows Amber (#F59E0B)

use crate::ui::theme;
use dioxus::prelude::*;
use std::collections::VecDeque;

/// Maximum number of trace entries retained in history.
pub const MAX_TRACE_ENTRIES: usize = 256;

// ─── Handshake Stage ───────────────────────────────────────────────────────────

/// The discrete stages of a network handshake traced by EdgeRay.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HandshakeStage {
    /// Initial DNS resolution for the target domain.
    DnsResolve,
    /// TCP three-way handshake to remote endpoint.
    TcpHandshake,
    /// TLS (or REALITY) cryptographic handshake.
    TlsReality,
    /// Full connection established successfully.
    Success,
    /// Connection failed (DPI block, timeout, or error).
    Failed,
}

impl HandshakeStage {
    /// Returns a human-readable label for the timeline node.
    pub fn label(&self) -> &'static str {
        match self {
            HandshakeStage::DnsResolve => "DNS RESOLVE",
            HandshakeStage::TcpHandshake => "TCP HANDSHAKE",
            HandshakeStage::TlsReality => "TLS / REALITY",
            HandshakeStage::Success => "SUCCESS",
            HandshakeStage::Failed => "FAILED",
        }
    }

    /// Returns the theme color for this stage.
    ///
    /// Success-path stages use Emerald; failure/interference uses Amber.
    pub fn color(&self) -> &'static str {
        match self {
            HandshakeStage::Success => theme::COLOR_EMERALD,
            HandshakeStage::Failed => theme::COLOR_AMBER,
            _ => theme::COLOR_ELECTRIC_CYAN,
        }
    }

    /// Returns an icon character/emoji for the timeline node.
    pub fn icon_char(&self) -> &'static str {
        match self {
            HandshakeStage::DnsResolve => "⟐",
            HandshakeStage::TcpHandshake => "⇄",
            HandshakeStage::TlsReality => "🔒",
            HandshakeStage::Success => "✓",
            HandshakeStage::Failed => "✗",
        }
    }

    /// Returns a 0-based ordinal for ordering stages on the timeline.
    pub fn ordinal(&self) -> u8 {
        match self {
            HandshakeStage::DnsResolve => 0,
            HandshakeStage::TcpHandshake => 1,
            HandshakeStage::TlsReality => 2,
            HandshakeStage::Success => 3,
            HandshakeStage::Failed => 3, // same level as Success (terminal)
        }
    }
}

// ─── Trace Status ──────────────────────────────────────────────────────────────

/// Outcome of a single handshake stage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TraceStatus {
    /// Stage completed successfully.
    Ok,
    /// Deep Packet Inspection interference detected.
    DpiDetected,
    /// Stage timed out before completing.
    Timeout,
    /// Stage failed with an unspecified error.
    Error,
}

impl TraceStatus {
    /// Maps the status to a CSS-class-ready color string.
    pub fn color(&self) -> &'static str {
        match self {
            TraceStatus::Ok => theme::COLOR_EMERALD,
            TraceStatus::DpiDetected => theme::COLOR_AMBER,
            TraceStatus::Timeout => theme::COLOR_AMBER,
            TraceStatus::Error => theme::COLOR_ERROR,
        }
    }

    /// Human-readable label for the status badge.
    pub fn label(&self) -> &'static str {
        match self {
            TraceStatus::Ok => "OK",
            TraceStatus::DpiDetected => "DPI",
            TraceStatus::Timeout => "TIMEOUT",
            TraceStatus::Error => "ERROR",
        }
    }

    /// Whether this status should trigger the amber/warning glow.
    pub fn is_warning(&self) -> bool {
        matches!(self, TraceStatus::DpiDetected | TraceStatus::Timeout)
    }
}

// ─── Trace Entry ───────────────────────────────────────────────────────────────

/// A single log entry in the handshake forensics timeline.
#[derive(Clone, Debug, PartialEq)]
pub struct TraceEntry {
    /// The handshake stage this entry represents.
    pub stage: HandshakeStage,
    /// Monotonic timestamp in milliseconds (relative to session start).
    pub timestamp_ms: u64,
    /// Measured latency for this stage in milliseconds.
    pub latency_ms: f32,
    /// Outcome of this stage.
    pub status: TraceStatus,
    /// Optional GeoIP label (e.g. "FRA", "SIN", "LAX").
    pub geo_label: Option<String>,
    /// Outbound tag identifier for routing correlation.
    pub outbound_tag: Option<String>,
}

impl TraceEntry {
    /// Returns the accent color for this entry based on its status.
    pub fn accent_color(&self) -> &'static str {
        if self.status == TraceStatus::Ok {
            self.stage.color()
        } else {
            self.status.color()
        }
    }

    /// SVG glow filter radius proportional to latency (higher latency → bigger pulse).
    pub fn pulse_radius(&self) -> f32 {
        let base = theme::TIMELINE_NODE_RADIUS;
        let max = theme::TIMELINE_PULSE_RADIUS;
        // Clamp latency contribution: 0ms → base, ≥200ms → max
        let t = (self.latency_ms / 200.0).clamp(0.0, 1.0);
        base + t * (max - base)
    }
}

// ─── Trace History ─────────────────────────────────────────────────────────────

/// Bounded ring buffer of trace entries. Old entries are discarded
/// when the buffer exceeds `capacity`.
#[derive(Clone, Debug)]
pub struct TraceHistory {
    /// Internal storage.
    pub entries: VecDeque<TraceEntry>,
    /// Maximum number of entries to retain.
    pub capacity: usize,
}

impl TraceHistory {
    /// Creates a new history with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Creates a history with the default capacity (`MAX_TRACE_ENTRIES`).
    pub fn with_default_capacity() -> Self {
        Self::new(MAX_TRACE_ENTRIES)
    }

    /// Pushes a new entry, evicting the oldest if at capacity.
    pub fn push(&mut self, entry: TraceEntry) {
        if self.entries.len() >= self.capacity {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    /// Number of entries currently stored.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the history is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns a reference to the most recently added entry.
    pub fn latest(&self) -> Option<&TraceEntry> {
        self.entries.back()
    }

    /// Returns a slice-like iterator over all retained entries (oldest first).
    pub fn entries_slice(&self) -> impl Iterator<Item = &TraceEntry> {
        self.entries.iter()
    }

    /// Compute the y-position of the `index`-th node on a vertical timeline
    /// of the given total `height`.
    pub fn node_y(&self, index: usize, padding_top: f32) -> f32 {
        padding_top + (index as f32 * theme::TIMELINE_NODE_SPACING)
    }

    /// Total height needed for all current entries.
    pub fn total_height(&self, padding_top: f32, padding_bottom: f32) -> f32 {
        if self.entries.is_empty() {
            return padding_top + padding_bottom;
        }
        padding_top
            + ((self.entries.len() - 1) as f32 * theme::TIMELINE_NODE_SPACING)
            + padding_bottom
    }
}

impl Default for TraceHistory {
    fn default() -> Self {
        Self::with_default_capacity()
    }
}

// ─── Component Props ───────────────────────────────────────────────────────────

/// Props for the ForensicsTracer component.
#[derive(Props, Clone, PartialEq)]
pub struct ForensicsTracerProps {
    /// The trace history to render.
    pub history: TraceHistory,

    /// Display width of the timeline panel in pixels.
    #[props(default = 360)]
    pub width: u32,

    /// Maximum visible height (scrollable if exceeded).
    #[props(default = 480)]
    pub max_height: u32,
}

impl PartialEq for TraceHistory {
    fn eq(&self, other: &Self) -> bool {
        self.capacity == other.capacity && self.entries == other.entries
    }
}

// ─── Component ─────────────────────────────────────────────────────────────────

/// Vertical timeline that traces the handshake lifecycle.
///
/// Each stage is represented as a node on a vertical SVG spine.
/// Nodes expand with a circular pulse animation on insertion.
/// Colors encode outcome: Emerald (success), Amber (DPI/timeout),
/// Red (error), Cyan (in-progress stages).
#[component]
pub fn ForensicsTracer(props: ForensicsTracerProps) -> Element {
    let w = props.width as f32;
    let padding_top = 30.0_f32;
    let padding_bottom = 30.0_f32;
    let spine_x = 50.0_f32;
    let total_h = props.history.total_height(padding_top, padding_bottom);
    let svg_h = total_h.max(props.max_height as f32);
    let entry_count = props.history.len();

    // CSS keyframes for the expanding pulse ring
    let pulse_keyframes = r#"
        @keyframes edgeray-trace-pulse {
            0%   { r: 10; opacity: 0.8; }
            50%  { r: 22; opacity: 0.2; }
            100% { r: 10; opacity: 0.8; }
        }
        @keyframes edgeray-trace-fadein {
            0%   { opacity: 0; transform: translateX(-8px); }
            100% { opacity: 1; transform: translateX(0); }
        }
    "#;

    rsx! {
        div {
            class: "relative overflow-y-auto",
            style: "max-height: {props.max_height}px;",

            style { {pulse_keyframes} }

            svg {
                class: "block",
                width: "{w}",
                height: "{svg_h}",
                view_box: "0 0 {w} {svg_h}",

                // ── Vertical spine ──
                line {
                    x1: "{spine_x}",
                    y1: "{padding_top}",
                    x2: "{spine_x}",
                    y2: "{padding_top + ((entry_count.max(1) - 1) as f32 * theme::TIMELINE_NODE_SPACING)}",
                    stroke: "rgba(255,255,255,0.10)",
                    stroke_width: "{theme::TIMELINE_SPINE_WIDTH}",
                    stroke_dasharray: "4 4",
                }

                // ── Stage nodes ──
                for (idx , entry) in props.history.entries_slice().enumerate() {
                    {
                        let cy = props.history.node_y(idx, padding_top);
                        let color = entry.accent_color();
                        let pulse_r = entry.pulse_radius();
                        let label = entry.stage.label();
                        let icon = entry.stage.icon_char();
                        let latency_str = format!("{:.0}ms", entry.latency_ms);
                        let status_label = entry.status.label();
                        let status_color = entry.status.color();
                        let geo = entry.geo_label.as_deref().unwrap_or("");
                        let label_x = spine_x + 36.0;
                        let anim_delay = format!("{}ms", idx * 80);

                        rsx! {
                            // Ambient pulse ring
                            circle {
                                cx: "{spine_x}",
                                cy: "{cy}",
                                r: "{pulse_r}",
                                fill: "none",
                                stroke: "{color}",
                                stroke_width: "1",
                                opacity: "0.3",
                                style: "transform-origin: {spine_x}px {cy}px; animation: edgeray-trace-pulse 2s ease-in-out infinite; animation-delay: {anim_delay};",
                            }

                            // Solid node
                            circle {
                                cx: "{spine_x}",
                                cy: "{cy}",
                                r: "{theme::TIMELINE_NODE_RADIUS}",
                                fill: "{color}",
                                opacity: "0.85",
                            }

                            // Icon text inside node
                            text {
                                x: "{spine_x}",
                                y: "{cy + 1.0}",
                                text_anchor: "middle",
                                dominant_baseline: "central",
                                fill: "white",
                                font_size: "9",
                                style: "pointer-events: none;",
                                "{icon}"
                            }

                            // Stage label
                            text {
                                x: "{label_x}",
                                y: "{cy - 8.0}",
                                fill: "{color}",
                                font_size: "11",
                                font_weight: "600",
                                font_family: "Inter, sans-serif",
                                style: "animation: edgeray-trace-fadein 0.3s ease-out; animation-delay: {anim_delay}; animation-fill-mode: both;",
                                "{label}"
                            }

                            // Latency + status badge
                            text {
                                x: "{label_x}",
                                y: "{cy + 8.0}",
                                fill: "rgba(255,255,255,0.50)",
                                font_size: "10",
                                font_family: "'JetBrains Mono', monospace",
                                style: "animation: edgeray-trace-fadein 0.3s ease-out; animation-delay: {anim_delay}; animation-fill-mode: both;",
                                "{latency_str}"
                            }

                            // Status badge
                            text {
                                x: "{label_x + 50.0}",
                                y: "{cy + 8.0}",
                                fill: "{status_color}",
                                font_size: "9",
                                font_weight: "700",
                                font_family: "'JetBrains Mono', monospace",
                                "{status_label}"
                            }

                            // Geo tag (if present)
                            if !geo.is_empty() {
                                text {
                                    x: "{label_x + 90.0}",
                                    y: "{cy + 8.0}",
                                    fill: "rgba(255,255,255,0.30)",
                                    font_size: "9",
                                    font_family: "'JetBrains Mono', monospace",
                                    "📍 {geo}"
                                }
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

    #[test]
    fn test_stage_labels() {
        assert_eq!(HandshakeStage::DnsResolve.label(), "DNS RESOLVE");
        assert_eq!(HandshakeStage::TcpHandshake.label(), "TCP HANDSHAKE");
        assert_eq!(HandshakeStage::TlsReality.label(), "TLS / REALITY");
        assert_eq!(HandshakeStage::Success.label(), "SUCCESS");
        assert_eq!(HandshakeStage::Failed.label(), "FAILED");
    }

    #[test]
    fn test_stage_colors() {
        assert_eq!(HandshakeStage::Success.color(), theme::COLOR_EMERALD);
        assert_eq!(HandshakeStage::Failed.color(), theme::COLOR_AMBER);
        assert_eq!(
            HandshakeStage::DnsResolve.color(),
            theme::COLOR_ELECTRIC_CYAN
        );
        assert_eq!(
            HandshakeStage::TcpHandshake.color(),
            theme::COLOR_ELECTRIC_CYAN
        );
        assert_eq!(
            HandshakeStage::TlsReality.color(),
            theme::COLOR_ELECTRIC_CYAN
        );
    }

    #[test]
    fn test_stage_ordinals() {
        assert_eq!(HandshakeStage::DnsResolve.ordinal(), 0);
        assert_eq!(HandshakeStage::TcpHandshake.ordinal(), 1);
        assert_eq!(HandshakeStage::TlsReality.ordinal(), 2);
        assert_eq!(HandshakeStage::Success.ordinal(), 3);
        assert_eq!(HandshakeStage::Failed.ordinal(), 3);
    }

    #[test]
    fn test_trace_status_colors() {
        assert_eq!(TraceStatus::Ok.color(), theme::COLOR_EMERALD);
        assert_eq!(TraceStatus::DpiDetected.color(), theme::COLOR_AMBER);
        assert_eq!(TraceStatus::Timeout.color(), theme::COLOR_AMBER);
        assert_eq!(TraceStatus::Error.color(), theme::COLOR_ERROR);
    }

    #[test]
    fn test_trace_status_is_warning() {
        assert!(!TraceStatus::Ok.is_warning());
        assert!(TraceStatus::DpiDetected.is_warning());
        assert!(TraceStatus::Timeout.is_warning());
        assert!(!TraceStatus::Error.is_warning());
    }

    #[test]
    fn test_trace_entry_accent_color() {
        let ok_entry = TraceEntry {
            stage: HandshakeStage::Success,
            timestamp_ms: 100,
            latency_ms: 10.0,
            status: TraceStatus::Ok,
            geo_label: None,
            outbound_tag: None,
        };
        assert_eq!(ok_entry.accent_color(), theme::COLOR_EMERALD);

        let dpi_entry = TraceEntry {
            stage: HandshakeStage::TlsReality,
            timestamp_ms: 200,
            latency_ms: 50.0,
            status: TraceStatus::DpiDetected,
            geo_label: None,
            outbound_tag: None,
        };
        assert_eq!(dpi_entry.accent_color(), theme::COLOR_AMBER);
    }

    #[test]
    fn test_pulse_radius_range() {
        let low = TraceEntry {
            stage: HandshakeStage::DnsResolve,
            timestamp_ms: 0,
            latency_ms: 0.0,
            status: TraceStatus::Ok,
            geo_label: None,
            outbound_tag: None,
        };
        assert!((low.pulse_radius() - theme::TIMELINE_NODE_RADIUS).abs() < f32::EPSILON);

        let high = TraceEntry {
            stage: HandshakeStage::DnsResolve,
            timestamp_ms: 0,
            latency_ms: 999.0,
            status: TraceStatus::Ok,
            geo_label: None,
            outbound_tag: None,
        };
        assert!((high.pulse_radius() - theme::TIMELINE_PULSE_RADIUS).abs() < f32::EPSILON);
    }

    #[test]
    fn test_trace_history_push_and_capacity() {
        let mut hist = TraceHistory::new(3);
        for i in 0..5 {
            hist.push(TraceEntry {
                stage: HandshakeStage::DnsResolve,
                timestamp_ms: i as u64 * 100,
                latency_ms: i as f32,
                status: TraceStatus::Ok,
                geo_label: None,
                outbound_tag: None,
            });
        }
        assert_eq!(hist.len(), 3);
        assert_eq!(hist.latest().unwrap().timestamp_ms, 400);
        assert_eq!(hist.entries.front().unwrap().timestamp_ms, 200);
    }

    #[test]
    fn test_trace_history_empty() {
        let hist = TraceHistory::new(10);
        assert!(hist.is_empty());
        assert!(hist.latest().is_none());
    }

    #[test]
    fn test_trace_history_default_capacity() {
        let hist = TraceHistory::with_default_capacity();
        assert_eq!(hist.capacity, MAX_TRACE_ENTRIES);
    }

    #[test]
    fn test_trace_history_total_height() {
        let mut hist = TraceHistory::new(10);
        // Empty: just padding
        assert!((hist.total_height(30.0, 30.0) - 60.0).abs() < f32::EPSILON);

        // One entry: no spacing, just top + bottom padding
        hist.push(TraceEntry {
            stage: HandshakeStage::DnsResolve,
            timestamp_ms: 0,
            latency_ms: 5.0,
            status: TraceStatus::Ok,
            geo_label: None,
            outbound_tag: None,
        });
        assert!((hist.total_height(30.0, 30.0) - 60.0).abs() < f32::EPSILON);

        // Two entries: top + 1 * spacing + bottom
        hist.push(TraceEntry {
            stage: HandshakeStage::TcpHandshake,
            timestamp_ms: 10,
            latency_ms: 12.0,
            status: TraceStatus::Ok,
            geo_label: None,
            outbound_tag: None,
        });
        let expected = 30.0 + theme::TIMELINE_NODE_SPACING + 30.0;
        assert!((hist.total_height(30.0, 30.0) - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn test_node_y_positions() {
        let hist = TraceHistory::new(10);
        let y0 = hist.node_y(0, 30.0);
        let y1 = hist.node_y(1, 30.0);
        assert!((y0 - 30.0).abs() < f32::EPSILON);
        assert!((y1 - (30.0 + theme::TIMELINE_NODE_SPACING)).abs() < f32::EPSILON);
    }
}
