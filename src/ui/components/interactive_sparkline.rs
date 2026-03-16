//! Interactive Sparkline Component
//!
//! A real-time sparkline chart using a `VecDeque<f32>` circular buffer of 60 points.
//! Draws the sparkline as an SVG `<polyline>` with a linear gradient stroke.
//! Designed for the Obsidian Engine telemetry panels.

use crate::ui::theme;
use dioxus::prelude::*;
use std::collections::VecDeque;

/// Maximum number of data points in the circular buffer.
pub const SPARKLINE_BUFFER_SIZE: usize = 60;

/// A circular buffer for sparkline data that automatically discards
/// old points when the capacity limit is reached.
#[derive(Clone, Debug, PartialEq)]
pub struct SparklineBuffer {
    /// Internal ring buffer storing the most recent data points
    pub data: VecDeque<f32>,
    /// Maximum number of points to retain
    pub capacity: usize,
}

impl SparklineBuffer {
    /// Creates a new buffer with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            data: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Creates a buffer using the default `SPARKLINE_BUFFER_SIZE`.
    pub fn with_default_capacity() -> Self {
        Self::new(SPARKLINE_BUFFER_SIZE)
    }

    /// Pushes a new value into the buffer. If the buffer is full,
    /// the oldest point is discarded first.
    pub fn push(&mut self, value: f32) {
        if self.data.len() >= self.capacity {
            self.data.pop_front();
        }
        self.data.push_back(value);
    }

    /// Returns the number of points currently stored.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns the most recent value, if any.
    pub fn latest(&self) -> Option<f32> {
        self.data.back().copied()
    }

    /// Returns the maximum value in the buffer, or 0.0 if empty.
    pub fn max_value(&self) -> f32 {
        self.data
            .iter()
            .copied()
            .fold(f32::NEG_INFINITY, f32::max)
            .max(0.0)
    }

    /// Returns the minimum value in the buffer, or 0.0 if empty.
    pub fn min_value(&self) -> f32 {
        self.data
            .iter()
            .copied()
            .fold(f32::INFINITY, f32::min)
            .min(0.0)
    }

    /// Converts the buffer contents into SVG polyline points.
    ///
    /// Maps the data into a coordinate system of `width` × `height` pixels,
    /// with the oldest point at x=0 and the newest point at x=width.
    pub fn to_polyline_points(&self, width: f32, height: f32) -> String {
        if self.data.is_empty() {
            return String::new();
        }

        let max = self.max_value().max(1.0); // avoid division by zero
        let count = self.data.len();
        let step_x = if count > 1 {
            width / (count - 1) as f32
        } else {
            0.0
        };

        let mut points = String::with_capacity(count * 16);
        for (i, &value) in self.data.iter().enumerate() {
            let x = i as f32 * step_x;
            let y = height - (value / max * height).clamp(0.0, height);
            if !points.is_empty() {
                points.push(' ');
            }
            points.push_str(&format!("{:.1},{:.1}", x, y));
        }

        points
    }

    /// Generates an SVG polygon fill area (the polyline closed to the bottom axis).
    pub fn to_area_points(&self, width: f32, height: f32) -> String {
        if self.data.is_empty() {
            return String::new();
        }

        let line_points = self.to_polyline_points(width, height);
        let count = self.data.len();
        let last_x = if count > 1 { width } else { 0.0 };

        format!(
            "0,{h} {line} {lx},{h}",
            h = height,
            line = line_points,
            lx = last_x
        )
    }
}

impl Default for SparklineBuffer {
    fn default() -> Self {
        Self::with_default_capacity()
    }
}

/// Props for the InteractiveSparkline component.
#[derive(Props, Clone, PartialEq)]
pub struct InteractiveSparklineProps {
    /// The circular data buffer
    pub buffer: SparklineBuffer,

    /// Display width in pixels
    #[props(default = 240)]
    pub width: u32,

    /// Display height in pixels
    #[props(default = 48)]
    pub height: u32,

    /// SVG gradient start color (left/old data)
    #[props(default = String::from("rgba(0, 242, 255, 0.05)"))]
    pub gradient_start: String,

    /// SVG gradient end color (right/current data)
    #[props(default = String::from("rgba(0, 242, 255, 0.30)"))]
    pub gradient_end: String,

    /// Stroke color for the line
    #[props(default = String::from(theme::COLOR_ELECTRIC_CYAN))]
    pub stroke_color: String,

    /// Label displayed beneath the sparkline
    #[props(default)]
    pub label: Option<String>,

    /// Current value displayed as monospace text
    #[props(default)]
    pub current_value: Option<String>,

    /// Unit suffix (e.g. "Mbps", "ms")
    #[props(default)]
    pub unit: Option<String>,
}

/// A real-time interactive sparkline with gradient fill and telemetry value display.
///
/// Features:
/// - SVG `<polyline>` rendering from a `VecDeque<f32>` circular buffer
/// - Linear gradient stroke from transparent to vibrant
/// - GPU-accelerated transitions via `will-change-transform`
/// - JetBrains Mono font for numeric readout
#[component]
pub fn InteractiveSparkline(props: InteractiveSparklineProps) -> Element {
    let w = props.width as f32;
    let h = props.height as f32;

    let polyline_points = props.buffer.to_polyline_points(w, h);
    let area_points = props.buffer.to_area_points(w, h);

    // Unique gradient ID based on component identity
    let grad_id = "edgeray-spark-grad";
    let area_grad_id = "edgeray-spark-area";

    rsx! {
        div { class: "flex flex-col gap-1 {theme::GPU_ACCELERATED}",

            // Value display
            if props.current_value.is_some() || props.label.is_some() {
                div { class: "flex items-baseline justify-between px-1",
                    if let Some(ref label) = props.label {
                        span { class: "{theme::TELEMETRY_LABEL}", "{label}" }
                    }
                    if let Some(ref value) = props.current_value {
                        span { class: "{theme::TELEMETRY_VALUE} text-lg",
                            style: "color: {props.stroke_color};",
                            "{value}"
                            if let Some(ref unit) = props.unit {
                                span { class: "text-xs text-white/40 ml-1", "{unit}" }
                            }
                        }
                    }
                }
            }

            // SVG chart
            svg {
                class: "block",
                width: "{props.width}",
                height: "{props.height}",
                view_box: "0 0 {props.width} {props.height}",
                preserve_aspect_ratio: "none",

                defs {
                    // Line gradient (left transparent → right vibrant)
                    linearGradient {
                        id: "{grad_id}",
                        x1: "0%",
                        y1: "0%",
                        x2: "100%",
                        y2: "0%",

                        stop { offset: "0%", stop_color: "{props.stroke_color}", stop_opacity: "0.2" }
                        stop { offset: "100%", stop_color: "{props.stroke_color}", stop_opacity: "1.0" }
                    }

                    // Area fill gradient (left transparent → right colored)
                    linearGradient {
                        id: "{area_grad_id}",
                        x1: "0%",
                        y1: "0%",
                        x2: "100%",
                        y2: "0%",

                        stop { offset: "0%", stop_color: "{props.gradient_start}" }
                        stop { offset: "100%", stop_color: "{props.gradient_end}" }
                    }
                }

                // Filled area below the line
                if !area_points.is_empty() {
                    polygon {
                        points: "{area_points}",
                        fill: "url(#{area_grad_id})",
                    }
                }

                // The sparkline itself
                if !polyline_points.is_empty() {
                    polyline {
                        points: "{polyline_points}",
                        fill: "none",
                        stroke: "url(#{grad_id})",
                        stroke_width: "1.5",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_push_and_capacity() {
        let mut buf = SparklineBuffer::new(5);
        for i in 0..10 {
            buf.push(i as f32);
        }
        assert_eq!(buf.len(), 5, "Buffer should cap at capacity");
        assert_eq!(buf.data[0], 5.0, "Oldest retained value should be 5.0");
        assert_eq!(buf.latest(), Some(9.0), "Latest value should be 9.0");
    }

    #[test]
    fn test_buffer_discards_oldest() {
        let mut buf = SparklineBuffer::new(3);
        buf.push(1.0);
        buf.push(2.0);
        buf.push(3.0);
        assert_eq!(buf.len(), 3);

        buf.push(4.0);
        assert_eq!(buf.len(), 3);
        assert_eq!(
            buf.data[0], 2.0,
            "First element should now be 2.0 after discard"
        );
        assert_eq!(buf.data[1], 3.0);
        assert_eq!(buf.data[2], 4.0);
    }

    #[test]
    fn test_buffer_max_min() {
        let mut buf = SparklineBuffer::new(10);
        buf.push(5.0);
        buf.push(2.0);
        buf.push(8.0);
        buf.push(1.0);
        assert!((buf.max_value() - 8.0).abs() < f32::EPSILON);
        assert!((buf.min_value() - 0.0).abs() < f32::EPSILON); // min_value returns min(actual_min, 0.0)
    }

    #[test]
    fn test_buffer_empty() {
        let buf = SparklineBuffer::new(10);
        assert!(buf.is_empty());
        assert_eq!(buf.latest(), None);
        assert!(buf.to_polyline_points(100.0, 50.0).is_empty());
    }

    #[test]
    fn test_polyline_single_point() {
        let mut buf = SparklineBuffer::new(10);
        buf.push(5.0);
        let points = buf.to_polyline_points(100.0, 50.0);
        // Single point at x=0, y = height - (value/max * height) = 50 - 50 = 0
        assert!(
            points.contains("0.0,0.0"),
            "Single point at origin, got: {points}"
        );
    }

    #[test]
    fn test_polyline_two_points() {
        let mut buf = SparklineBuffer::new(10);
        buf.push(0.0);
        buf.push(10.0);
        let points = buf.to_polyline_points(100.0, 50.0);
        // First point: x=0, y=50 (value 0 → bottom)
        // Second point: x=100, y=0 (value 10 → top)
        assert!(
            points.contains("0.0,50.0"),
            "First point at bottom-left, got: {points}"
        );
        assert!(
            points.contains("100.0,0.0"),
            "Second point at top-right, got: {points}"
        );
    }

    #[test]
    fn test_default_buffer_capacity() {
        let buf = SparklineBuffer::with_default_capacity();
        assert_eq!(buf.capacity, SPARKLINE_BUFFER_SIZE);
    }

    #[test]
    fn test_area_points_non_empty() {
        let mut buf = SparklineBuffer::new(10);
        buf.push(5.0);
        buf.push(10.0);
        let area = buf.to_area_points(100.0, 50.0);
        assert!(!area.is_empty());
        // Area polygon should start at bottom-left (0,height)
        assert!(
            area.starts_with("0,50"),
            "Area should start at bottom-left, got: {area}"
        );
    }
}
