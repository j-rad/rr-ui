//! CPU Gauge Component

use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct CpuGaugeProps {
    pub percentage: f64,
    #[props(default = 120)]
    pub size: i32,
}

#[component]
pub fn CpuGauge(props: CpuGaugeProps) -> Element {
    let percentage = props.percentage.min(100.0).max(0.0);
    let angle = (percentage / 100.0) * 270.0 - 135.0;

    let color = if percentage < 50.0 {
        "#10b981" // green
    } else if percentage < 80.0 {
        "#f59e0b" // orange
    } else {
        "#ef4444" // red
    };

    rsx! {
        div { class: "relative flex items-center justify-center",
            style: "width: {props.size}px; height: {props.size}px;",

            // SVG Gauge
            svg {
                width: "{props.size}",
                height: "{props.size}",
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
                    stroke: "{color}",
                    stroke_width: "8",
                    stroke_linecap: "round",
                    stroke_dasharray: "188.5",
                    stroke_dashoffset: "{188.5 * (1.0 - percentage / 100.0)}",
                    style: "transition: stroke-dashoffset 0.5s ease;",
                }
            }

            // Center text
            div { class: "absolute inset-0 flex flex-col items-center justify-center",
                div { class: "text-2xl font-bold text-white", "{percentage:.1}%" }
                div { class: "text-xs text-gray-500", "CPU" }
            }
        }
    }
}
