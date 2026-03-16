//! Sparkline Component
//!
//! A lightweight real-time chart for displaying traffic history.
//! Designed for 60FPS updates with minimal re-renders.

use dioxus::prelude::*;
use std::collections::VecDeque;

#[derive(Props, Clone, PartialEq)]
pub struct SparklineProps {
    /// Historical data points (up, down) in bytes
    pub data: VecDeque<(i64, i64)>,
    /// Width in pixels
    #[props(default = 200)]
    pub width: u32,
    /// Height in pixels
    #[props(default = 40)]
    pub height: u32,
    /// Whether to show upload line
    #[props(default = true)]
    pub show_upload: bool,
    /// Whether to show download line
    #[props(default = true)]
    pub show_download: bool,
    /// Fill color for upload area
    #[props(default)]
    pub upload_color: Option<String>,
    /// Fill color for download area
    #[props(default)]
    pub download_color: Option<String>,
}

/// Generate SVG path from data points
fn generate_path(data: &[(i64, i64)], width: u32, height: u32, use_upload: bool) -> String {
    if data.is_empty() {
        return String::new();
    }

    // Find max value for scaling
    let max_val = data
        .iter()
        .map(|(up, down)| if use_upload { *up } else { *down })
        .max()
        .unwrap_or(1)
        .max(1); // Avoid division by zero

    let step_x = width as f64 / (data.len().max(1) - 1).max(1) as f64;
    let scale_y = height as f64 / max_val as f64;

    let mut path = String::with_capacity(data.len() * 20);

    for (i, (up, down)) in data.iter().enumerate() {
        let value = if use_upload { *up } else { *down };
        let x = i as f64 * step_x;
        let y = height as f64 - (value as f64 * scale_y).min(height as f64);

        if i == 0 {
            path.push_str(&format!("M {:.1} {:.1}", x, y));
        } else {
            path.push_str(&format!(" L {:.1} {:.1}", x, y));
        }
    }

    path
}

/// Generate area path (path closed to bottom)
fn generate_area_path(data: &[(i64, i64)], width: u32, height: u32, use_upload: bool) -> String {
    if data.is_empty() {
        return String::new();
    }

    let line_path = generate_path(data, width, height, use_upload);
    if line_path.is_empty() {
        return line_path;
    }

    let step_x = width as f64 / (data.len().max(1) - 1).max(1) as f64;
    let last_x = (data.len() - 1) as f64 * step_x;

    format!("{} L {:.1} {} L 0 {} Z", line_path, last_x, height, height)
}

/// Real-time sparkline chart for traffic visualization
#[component]
pub fn Sparkline(props: SparklineProps) -> Element {
    let data: Vec<_> = props.data.iter().copied().collect();
    let width = props.width;
    let height = props.height;

    let upload_color = props
        .upload_color
        .as_deref()
        .unwrap_or("rgba(59, 130, 246, 0.5)");
    let download_color = props
        .download_color
        .as_deref()
        .unwrap_or("rgba(34, 197, 94, 0.5)");

    let upload_line_color = "rgb(59, 130, 246)"; // blue-500
    let download_line_color = "rgb(34, 197, 94)"; // green-500

    rsx! {
        div { class: "sparkline-container relative",
            svg {
                width: "{width}",
                height: "{height}",
                view_box: "0 0 {width} {height}",

                // Download area (green, behind)
                if props.show_download && !data.is_empty() {
                    path {
                        d: "{generate_area_path(&data, width, height, false)}",
                        fill: "{download_color}",
                    }
                }

                // Upload area (blue, in front)
                if props.show_upload && !data.is_empty() {
                    path {
                        d: "{generate_area_path(&data, width, height, true)}",
                        fill: "{upload_color}",
                    }
                }

                // Download line
                if props.show_download && !data.is_empty() {
                    path {
                        d: "{generate_path(&data, width, height, false)}",
                        fill: "none",
                        stroke: "{download_line_color}",
                        stroke_width: "1.5",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                    }
                }

                // Upload line
                if props.show_upload && !data.is_empty() {
                    path {
                        d: "{generate_path(&data, width, height, true)}",
                        fill: "none",
                        stroke: "{upload_line_color}",
                        stroke_width: "1.5",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                    }
                }
            }
        }
    }
}

/// Mini sparkline for dashboard cards
#[derive(Props, Clone, PartialEq)]
pub struct MiniSparklineProps {
    pub data: VecDeque<(i64, i64)>,
}

#[component]
pub fn MiniSparkline(props: MiniSparklineProps) -> Element {
    rsx! {
        Sparkline {
            data: props.data,
            width: 120,
            height: 32,
            show_upload: true,
            show_download: true,
        }
    }
}

/// Large sparkline with legend for detailed view
#[derive(Props, Clone, PartialEq)]
pub struct DetailSparklineProps {
    pub data: VecDeque<(i64, i64)>,
    /// Current upload rate in bytes/sec
    #[props(default)]
    pub upload_rate: Option<i64>,
    /// Current download rate in bytes/sec  
    #[props(default)]
    pub download_rate: Option<i64>,
}

/// Format bytes to human-readable string
pub fn format_rate(bytes_per_sec: i64) -> String {
    const KB: i64 = 1024;
    const MB: i64 = KB * 1024;
    const GB: i64 = MB * 1024;

    if bytes_per_sec >= GB {
        format!("{:.1} GB/s", bytes_per_sec as f64 / GB as f64)
    } else if bytes_per_sec >= MB {
        format!("{:.1} MB/s", bytes_per_sec as f64 / MB as f64)
    } else if bytes_per_sec >= KB {
        format!("{:.0} KB/s", bytes_per_sec as f64 / KB as f64)
    } else {
        format!("{} B/s", bytes_per_sec)
    }
}

#[component]
pub fn DetailSparkline(props: DetailSparklineProps) -> Element {
    rsx! {
        div { class: "space-y-2",
            // Legend
            div { class: "flex items-center justify-between text-xs",
                div { class: "flex items-center gap-4",
                    div { class: "flex items-center gap-1.5",
                        div { class: "w-2 h-2 rounded-full bg-blue-500" }
                        span { class: "text-gray-400", "Upload" }
                        if let Some(rate) = props.upload_rate {
                            span { class: "text-white font-medium", "{format_rate(rate)}" }
                        }
                    }
                    div { class: "flex items-center gap-1.5",
                        div { class: "w-2 h-2 rounded-full bg-green-500" }
                        span { class: "text-gray-400", "Download" }
                        if let Some(rate) = props.download_rate {
                            span { class: "text-white font-medium", "{format_rate(rate)}" }
                        }
                    }
                }
            }

            // Chart
            div { class: "bg-bg-tertiary rounded-lg p-2",
                Sparkline {
                    data: props.data,
                    width: 300,
                    height: 60,
                    show_upload: true,
                    show_download: true,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_path_empty() {
        let path = generate_path(&[], 100, 50, true);
        assert!(path.is_empty());
    }

    #[test]
    fn test_generate_path_single_point() {
        let data = [(100, 200)];
        let path = generate_path(&data, 100, 50, true);
        assert!(path.contains("M"));
    }

    #[test]
    fn test_format_rate() {
        assert_eq!(format_rate(512), "512 B/s");
        assert_eq!(format_rate(1024), "1 KB/s");
        assert_eq!(format_rate(1024 * 1024), "1.0 MB/s");
    }
}
