//! Memory Progress Bar Component

use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct MemoryProgressProps {
    pub used_gb: f64,
    pub total_gb: f64,
}

#[component]
pub fn MemoryProgress(props: MemoryProgressProps) -> Element {
    let percentage = (props.used_gb / props.total_gb * 100.0).min(100.0).max(0.0);

    let color = if percentage < 60.0 {
        "bg-green-500"
    } else if percentage < 85.0 {
        "bg-yellow-500"
    } else {
        "bg-red-500"
    };

    rsx! {
        div { class: "space-y-2",
            div { class: "flex items-center justify-between text-sm",
                span { class: "text-gray-400", "Memory Usage" }
                span { class: "text-white font-medium",
                    "{props.used_gb:.1} GB / {props.total_gb:.1} GB ({percentage:.0}%)"
                }
            }

            div { class: "h-2 bg-gray-800 rounded-full overflow-hidden",
                div {
                    class: "h-full {color} transition-all duration-500 ease-out",
                    style: "width: {percentage}%;",
                }
            }
        }
    }
}
