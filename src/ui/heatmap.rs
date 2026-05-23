// src/ui/components/heatmap.rs
use crate::models::MeshNode;
use dioxus::prelude::*;

#[component]
pub fn IspHealthHeatmap(nodes: Vec<MeshNode>) -> Element {
    rsx! {
        div { class: "bg-gray-800 p-6 rounded-lg",
            div { class: "flex justify-between items-center mb-6",
                h3 { class: "text-lg font-bold text-white", "Global ISP Health Heatmap" }
                div { class: "flex space-x-4 text-xs text-gray-400",
                    div { class: "flex items-center",
                        span { class: "w-3 h-3 bg-green-500 rounded-sm mr-2" }
                        "Excellent"
                    }
                    div { class: "flex items-center",
                        span { class: "w-3 h-3 bg-yellow-500 rounded-sm mr-2" }
                        "Fair"
                    }
                    div { class: "flex items-center",
                        span { class: "w-3 h-3 bg-red-500 rounded-sm mr-2" }
                        "Critical"
                    }
                }
            }

            div { class: "grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 xl:grid-cols-8 gap-4",
                for node in nodes.clone() {
                    NodeHeatSquare { node: node }
                }
            }

            div { class: "mt-6 pt-4 border-t border-gray-700 flex flex-col space-y-2 text-xs text-gray-500",
                div { class: "flex justify-between",
                    span { "Updated: real-time via ProbeTransport" }
                    span { "Target: CDN Edge + Domestic Hubs" }
                }
                if let Some(blocked) = detect_blockages(&nodes) {
                    div { class: "text-red-400 font-medium",
                        "Blockages detected: {blocked}"
                    }
                }
            }
        }
    }
}

fn detect_blockages(nodes: &[MeshNode]) -> Option<String> {
    let mut blocked_isps = std::collections::HashSet::new();
    for node in nodes {
        if node.health.packet_loss_percent > 10.0 || node.health.isp_score < 0.3 {
            // Simplified logic: mapping region to known ASNs or simulated lookup
            if node.region.contains("MCI") || node.name.contains("MCI") {
                blocked_isps.insert("AS44244 - MCI");
            } else if node.region.contains("Irancell") || node.name.contains("Irancell") {
                blocked_isps.insert("AS43604 - Irancell");
            } else {
                blocked_isps.insert("Unknown ASN");
            }
        }
    }
    if blocked_isps.is_empty() {
        None
    } else {
        Some(blocked_isps.into_iter().collect::<Vec<_>>().join(", "))
    }
}

#[component]
fn NodeHeatSquare(node: MeshNode) -> Element {
    let score = node.health.isp_score;
    let bg_color = if score >= 0.8 {
        "bg-green-500"
    } else if score >= 0.5 {
        "bg-yellow-500"
    } else if score > 0.0 {
        "bg-red-500"
    } else {
        "bg-gray-700"
    };

    let border_color = if score >= 0.8 {
        "border-green-400"
    } else if score >= 0.5 {
        "border-yellow-400"
    } else if score > 0.0 {
        "border-red-400"
    } else {
        "border-gray-600"
    };

    rsx! {
        div {
            class: "flex flex-col items-center space-y-2 p-2 rounded-lg border {border_color} bg-opacity-10 {bg_color} group cursor-help transition-all duration-300 hover:scale-105",
            title: "{node.name}: {score * 100.0:.1}% ISP Health",

            div { class: "w-full aspect-square rounded {bg_color} shadow-lg relative overflow-hidden",
                div { class: "absolute inset-0 bg-white opacity-0 group-hover:opacity-20 transition-opacity" }
                if score < 0.3 && score > 0.0 {
                    div { class: "absolute inset-0 animate-pulse bg-red-600 opacity-50" }
                }
            }

            span { class: "text-[10px] font-medium text-gray-300 truncate w-full text-center", "{node.name}" }
        }
    }
}
