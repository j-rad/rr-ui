// src/ui/pages/mesh.rs
//! Mesh Orchestration Dashboard

use crate::models::{MeshNode, MeshNodeRole, MeshNodeStatus};
use dioxus::prelude::*;

#[component]
pub fn MeshPage() -> Element {
    // Fetch nodes (auto-refresh every 5s if possible, or just load once)
    let mut nodes_resource: Resource<Result<Vec<MeshNode>, crate::ui::server_fns::ServerFnError>> =
        use_resource(move || async move { crate::ui::server_fns::get_mesh_nodes().await });

    let mut stats_resource: Resource<
        Result<crate::models::ClusterStats, crate::ui::server_fns::ServerFnError>,
    > = use_resource(move || async move { crate::ui::server_fns::get_cluster_stats().await });

    let nodes: Vec<MeshNode> = nodes_resource
        .cloned()
        .and_then(|r| r.ok())
        .unwrap_or_default();
    let stats: crate::models::ClusterStats = stats_resource
        .cloned()
        .and_then(|r| r.ok())
        .unwrap_or_default();

    let online_nodes = stats.online_nodes;
    let total_clients = stats.total_clients;
    let total_nodes = stats.total_nodes;

    let handle_refresh = move |_: Event<MouseData>| {
        nodes_resource.restart();
        stats_resource.restart();
    };

    let handle_disconnect = move |name: String| {
        spawn(async move {
            if let Ok(_) = crate::ui::server_fns::remove_mesh_node(name).await {
                // Refresh list
                nodes_resource.restart();
                stats_resource.restart();
            }
        });
    };

    rsx! {
        div { class: "p-6 space-y-6",
            // Header Stats
            div { class: "grid grid-cols-1 md:grid-cols-3 gap-6",
                div { class: "bg-gray-800 p-6 rounded-lg",
                    h3 { class: "text-gray-400 text-sm font-medium", "Total Nodes" }
                    p { class: "text-3xl font-bold text-white mt-2", "{total_nodes}" }
                }
                div { class: "bg-gray-800 p-6 rounded-lg",
                    h3 { class: "text-gray-400 text-sm font-medium", "Online Nodes" }
                    p { class: "text-3xl font-bold text-green-500 mt-2", "{online_nodes}" }
                }
                div { class: "bg-gray-800 p-6 rounded-lg",
                    h3 { class: "text-gray-400 text-sm font-medium", "Total Clients" }
                    p { class: "text-3xl font-bold text-blue-500 mt-2", "{total_clients}" }
                }
            }

            // Node Grid
            div { class: "grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-6",
                for node in nodes {
                    MeshNodeCard {
                        node: node.clone(),
                        on_disconnect: handle_disconnect.clone()
                    }
                }

                // Add Node Button
                div {
                    class: "bg-gray-800 bg-opacity-50 p-6 rounded-lg border-2 border-dashed border-gray-700 flex flex-col items-center justify-center cursor-pointer hover:border-gray-500 hover:bg-gray-700 transition h-48",
                    // onclick: move |_| show_add_modal.set(true),

                    div { class: "w-12 h-12 bg-gray-700 rounded-full flex items-center justify-center mb-3",
                        i { class: "fas fa-plus text-gray-400" }
                    }
                    span { class: "text-gray-400 font-medium", "Connect New Node" }
                }
            }
        }
    }
}

#[component]
fn MeshNodeCard(node: MeshNode, on_disconnect: EventHandler<String>) -> Element {
    let status_color = match node.status {
        MeshNodeStatus::Online => "text-green-500",
        MeshNodeStatus::Offline => "text-red-500",
        MeshNodeStatus::Syncing => "text-yellow-500",
        MeshNodeStatus::Degraded => "text-orange-500",
        MeshNodeStatus::Unknown => "text-gray-500",
        MeshNodeStatus::Maintenance => "text-blue-500",
    };

    let status_dot = match node.status {
        MeshNodeStatus::Online => "bg-green-500",
        MeshNodeStatus::Offline => "bg-red-500",
        MeshNodeStatus::Syncing => "bg-yellow-500",
        MeshNodeStatus::Degraded => "bg-orange-500",
        MeshNodeStatus::Unknown => "bg-gray-500",
        MeshNodeStatus::Maintenance => "bg-blue-500",
    };

    rsx! {
        div { class: "bg-gray-800 rounded-lg p-6 relative overflow-hidden group hover:shadow-lg transition-all duration-300",
            // Status Indicator Stripe
            div { class: "absolute top-0 left-0 w-1 h-full {status_dot}" }

            div { class: "flex justify-between items-start mb-4 pl-3",
                div {
                    div { class: "flex items-center space-x-2",
                        h3 { class: "text-lg font-bold text-white", "{node.name}" }
                        if node.is_local {
                            span { class: "px-2 py-0.5 bg-blue-900 text-blue-200 text-xs rounded-full", "Local" }
                        }
                    }
                    p { class: "text-gray-400 text-sm mt-1", "{node.address}" }
                }
                div { class: "flex flex-col items-end",
                    span { class: "text-sm font-medium {status_color} flex items-center",
                        span { class: "w-2 h-2 rounded-full {status_dot} mr-2 inline-block" }
                        "{node.status:?}"
                    }
                    span { class: "text-xs text-gray-500 mt-1", "{node.role:?}" }
                }
            }

            div { class: "grid grid-cols-2 gap-4 pl-3 mb-4",
                div {
                    p { class: "text-xs text-gray-500 uppercase", "Clients" }
                    p { class: "text-lg font-semibold text-white", "{node.client_count}" }
                }
                div {
                    p { class: "text-xs text-gray-500 uppercase", "Version" }
                    p { class: "text-lg font-semibold text-white", "{node.version.as_deref().unwrap_or(\"v1.0.0\")}" }
                }
            }

            // Actions
            div { class: "flex justify-end space-x-2 pl-3 pt-3 border-t border-gray-700",
                button { class: "p-2 text-gray-400 hover:text-white transition", title: "Sync Config",
                    i { class: "fas fa-sync-alt" }
                }
                button { class: "p-2 text-gray-400 hover:text-white transition", title: "View Logs",
                    i { class: "fas fa-file-alt" }
                }
                if !node.is_local {
                     button { class: "p-2 text-gray-400 hover:text-red-400 transition", title: "Disconnect",
                        onclick: move |_| on_disconnect.call(node.name.clone()),
                        i { class: "fas fa-unlink" }
                    }
                }
            }
        }
    }
}
