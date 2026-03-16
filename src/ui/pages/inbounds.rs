//! Inbounds Page

use crate::domain::models::Client;
use crate::ui::components::card::Card;
use crate::ui::components::inbound_modal::InboundModal;
use crate::ui::components::qr_code_modal::QrCodeModal;
use crate::ui::server_fns::ServerFnError;
#[cfg(feature = "web")]
use crate::ui::server_fns::{ClientOperation, list_inbounds, manage_client};
use dioxus::prelude::*;

#[component]
pub fn InboundsPage() -> Element {
    // Resources
    let inbounds_resource = use_resource(move || async move {
        #[cfg(feature = "web")]
        {
            list_inbounds().await
        }
        #[cfg(not(feature = "web"))]
        {
            Ok::<Vec<crate::models::Inbound<'static>>, ServerFnError>(vec![])
        }
    });

    // Modal state
    let mut inbound_modal_open = use_signal(|| false);
    let mut qr_modal_open = use_signal(|| false);
    let mut client_details_open = use_signal(|| false);
    let mut selected_inbound_id = use_signal(|| None::<i64>);

    // We store the full client object for the details modal
    let mut selected_client = use_signal(|| None::<(String, Client)>); // (inbound_id, client)
    let mut selected_inbound_for_details = use_signal(|| None::<crate::models::Inbound<'static>>);
    let mut expanded_inbound = use_signal(|| None::<i64>);
    let mut selected_client_email = use_signal(|| None::<String>);

    // UI state
    // Mock data for display (replace with connection_tracker/GlobalState later)
    let mock_clients = vec![crate::ui::components::client_table::ClientData {
        email: "user1@example.com".to_string(),
        uuid: "uuid-1234".to_string(),
        enabled: true,
        total_gb: Some(100),
        expiry_time: Some(1738415000000), // ~30 days from now
        up: 1024 * 1024 * 500,            // 500 MB
        down: 1024 * 1024 * 1500,         // 1.5 GB
        connection_url: Some(
            "vless://uuid-1234@example.com:8443?type=tcp&security=tls#Primary%20VLESS".to_string(),
        ),
        json_config: None,
    }];

    let open_add_inbound = move |_| {
        selected_inbound_id.set(None);
        inbound_modal_open.set(true);
    };

    let mut open_qr_code = move |_| {
        qr_modal_open.set(true);
    };

    let handle_client_manage = move |email: String| {
        selected_client_email.set(Some(email));
        client_details_open.set(true);
    };

    let mut toggle_expand = move |id: i64| {
        if expanded_inbound() == Some(id) {
            expanded_inbound.set(None);
        } else {
            expanded_inbound.set(Some(id));
        }
    };

    rsx! {
        div { class: "p-6 space-y-6",
            // Stats Row (unchanged)
            div { class: "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-4",
                Card {
                    div { class: "flex items-center gap-3",
                        div { class: "p-2 bg-blue-500/10 rounded-lg text-blue-400",
                        span { class: "material-symbols-outlined", "swap_vert" }
                        }
                        div {
                            div { class: "text-xs text-secondary font-medium uppercase tracking-wider", "Total Traffic" }
                            div { class: "text-lg font-bold text-white", "2.0 GB" }
                        }
                    }
                }
                Card {
                    div { class: "flex items-center gap-3",
                        div { class: "p-2 bg-green-500/10 rounded-lg text-green-400",
                            span { class: "material-symbols-outlined", "pie_chart" }
                        }
                        div {
                            div { class: "text-xs text-secondary font-medium uppercase tracking-wider", "Total Usage" }
                            div { class: "text-lg font-bold text-white", "0 B" }
                        }
                    }
                }
                Card {
                    div { class: "flex items-center gap-3",
                        div { class: "p-2 bg-purple-500/10 rounded-lg text-purple-400",
                            span { class: "material-symbols-outlined", "history" }
                        }
                        div {
                            div { class: "text-xs text-secondary font-medium uppercase tracking-wider", "All Time" }
                            div { class: "text-lg font-bold text-white", "0 B" }
                        }
                    }
                }
                 Card {
                    div { class: "flex items-center gap-3",
                        div { class: "p-2 bg-orange-500/10 rounded-lg text-orange-400",
                            span { class: "material-symbols-outlined", "dns" }
                        }
                        div {
                            div { class: "text-xs text-secondary font-medium uppercase tracking-wider", "Inbounds" }
                            div { class: "text-lg font-bold text-white", "1" }
                        }
                    }
                }
                 Card {
                    div { class: "flex items-center gap-3",
                        div { class: "p-2 bg-red-500/10 rounded-lg text-red-500",
                            span { class: "material-symbols-outlined", "group" }
                        }
                        div {
                            div { class: "text-xs text-secondary font-medium uppercase tracking-wider", "Clients" }
                            div { class: "text-lg font-bold text-white", "1" }
                        }
                    }
                }
            }

             // Header & Actions
            div { class: "bg-bg-panel border border-border rounded-lg shadow-sm flex flex-col min-h-[500px]",
                div { class: "p-4 border-b border-border flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4",
                    div { class: "flex items-center space-x-3",
                        button {
                            class: "flex items-center justify-center px-4 py-1.5 bg-primary hover:bg-primary-hover text-white text-sm font-medium rounded transition shadow-sm",
                            onclick: open_add_inbound,
                            span { class: "material-symbols-outlined text-[18px] mr-1.5", "add" }
                            "Add Inbound"
                        }
                        div { class: "relative",
                            button { class: "flex items-center justify-center px-4 py-1.5 bg-transparent hover:border-primary hover:text-primary border border-border text-gray-300 text-sm font-medium rounded transition",
                                span { class: "material-symbols-outlined text-[18px] mr-1.5", "playlist_add_check" }
                                "General Actions"
                                span { class: "material-symbols-outlined text-[18px] ml-1", "expand_more" }
                            }
                        }
                    }
                    div { class: "flex items-center space-x-4 text-gray-400",
                        button { class: "hover:text-white transition p-1 rounded hover:bg-white/5",
                            span { class: "material-symbols-outlined", "refresh" }
                        }
                    }
                }

                // Search Bar
                div { class: "p-4 border-b border-border flex items-center gap-3 bg-bg-panel",
                    span { class: "material-symbols-outlined text-gray-500 text-[20px]", "filter_alt" }
                    div { class: "relative flex-1 max-w-sm group",
                        input {
                            class: "w-full bg-[#0a0a0a] border border-border text-gray-300 text-sm rounded px-3 py-1.5 focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary placeholder-gray-600 transition-all",
                            placeholder: "Search",
                            r#type: "text"
                        }
                    }
                }

                // Table Header
                div { class: "grid grid-cols-12 gap-4 px-6 py-3 border-b border-border text-xs font-semibold text-gray-400 uppercase tracking-wider bg-[#1d1d1d]",
                     div { class: "col-span-1 text-center", "ID" }
                     div { class: "col-span-1 text-center", "Menu" }
                     div { class: "col-span-1 text-center", "Enabled" }
                     div { class: "col-span-2", "Remark" }
                     div { class: "col-span-1 text-center", "Port" }
                     div { class: "col-span-2", "Protocol" }
                     div { class: "col-span-1 text-center", "Clients" }
                     div { class: "col-span-2 text-center", "Traffic" }
                     div { class: "col-span-1 text-center", "Duration" }
                }

                // Table Body
                div { class: "divide-y divide-border text-sm text-gray-300",
                    // Mock Row 1
                    div { class: "group",
                        div {
                            class: "grid grid-cols-12 gap-4 px-6 py-4 items-center hover:bg-[#1f1f1f] transition-colors cursor-pointer",
                            onclick: move |_| toggle_expand(1),

                            div { class: "col-span-1 flex justify-center items-center gap-1",
                                span { class: "font-mono text-gray-500", "1" }
                            }
                             div { class: "col-span-1 flex justify-center",
                                button {
                                    class: "material-symbols-outlined text-gray-500 cursor-pointer hover:text-white transition-colors text-[20px]",
                                    onclick: move |e| {
                                        e.stop_propagation();
                                        open_qr_code(e);
                                    },
                                    "more_vert"
                                }
                            }
                            div { class: "col-span-1 flex justify-center",
                                 span { class: "material-symbols-outlined text-green-500", "check_circle" }
                            }
                            div { class: "col-span-2 font-medium text-white truncate", "Primary VLESS" }
                            div { class: "col-span-1 text-center text-gray-400 font-mono", "8443" }
                             div { class: "col-span-2 flex gap-1.5 flex-wrap",
                                span { class: "px-1.5 py-0.5 rounded text-[10px] bg-blue-900/20 text-blue-400 border border-blue-900/30", "vless" }
                                span { class: "px-1.5 py-0.5 rounded text-[10px] bg-blue-900/20 text-blue-400 border border-blue-900/30", "tls" }
                            }
                             div { class: "col-span-1 flex justify-center",
                                span { class: "w-5 h-5 flex items-center justify-center rounded-full border border-blue-500/30 bg-blue-500/10 text-blue-400 text-[10px]",
                                    "{mock_clients.len()}"
                                }
                            }
                             div { class: "col-span-2 flex justify-center",
                                 span { class: "px-2 py-0.5 rounded-full text-xs bg-black/30 text-gray-400 border border-border font-mono", "2.0 GB / ∞" }
                            }
                             div { class: "col-span-1 flex justify-center",
                                span {
                                    class: "material-symbols-outlined transition-transform duration-200",
                                    class: if expanded_inbound() == Some(1) { "rotate-180" } else { "" },
                                    "expand_more"
                                }
                            }
                        }

                        // Expandable Client Table
                        if expanded_inbound() == Some(1) {
                            crate::ui::components::client_table::ClientTable {
                                clients: mock_clients.clone(),
                                inbound_id: 1,
                                on_manage: handle_client_manage,
                            }
                        }
                    }
                }
            }

            // Modals
            InboundModal {
                open: inbound_modal_open,
                inbound_id: selected_inbound_id(),
            }

            QrCodeModal {
                open: qr_modal_open,
                connection_url: "vless://uuid-1234@example.com:8443?type=tcp&security=tls#Primary%20VLESS".to_string(),
                remark: "Primary VLESS".to_string(),
                json_config: None,
            }

            crate::ui::components::client_details_modal::ClientDetailsModal {
                open: client_details_open,
                client: Some(crate::domain::models::Client {
                    level: None,
                    email: Some("user1@example.com".to_string()),
                    enable: true,
                    id: Some("uuid-1234".to_string()),
                    flow: None,
                    limit_ip: Some(0),
                    total_flow_limit: 100,
                    expiry_time: 0,
                    tg_id: None,
                    sub_id: None,
                    next_reset_date: None,
                    inbound_tag: None,
                    reset: 0,
                    comment: None,
                    created_at: None,
                    updated_at: None,
                    up: 0,
                    down: 0,
                    extra: std::collections::HashMap::new(),
                    password: None,
                    up_speed_limit: 0,
                    down_speed_limit: 0,
                }),
                inbound: Some(crate::domain::models::Inbound {
                    id: None,
                    remark: std::borrow::Cow::Borrowed("Primary VLESS"),
                    enable: true,
                    listen: std::borrow::Cow::Borrowed("0.0.0.0"),
                    port: 8443,
                    protocol: crate::domain::models::InboundProtocol::Vless,
                    settings: crate::domain::models::ProtocolSettings::default(),
                    stream_settings: crate::domain::models::StreamSettings {
                        network: std::borrow::Cow::Borrowed("tcp"),
                        security: std::borrow::Cow::Borrowed("none"),
                        tcp_settings: Some(crate::domain::models::TcpSettings {
                            header_type: "none".to_string(),
                            request: None,
                            response: None,
                        }),
                        tls_settings: None,
                        reality_settings: None,
                        ws_settings: None,
                        http_settings: None,
                        kcp_settings: None,
                        grpc_settings: None,
                        mqtt_settings: None,
                        db_mimic_settings: None,
                        slipstream_settings: None,
                    },
                    tag: std::borrow::Cow::Borrowed("inbound-1"),
                    sniffing: crate::domain::models::Sniffing {
                        enabled: true,
                        dest_override: Some(vec![std::borrow::Cow::Borrowed("http"), std::borrow::Cow::Borrowed("tls")]),
                        metadata_only: false,
                        route_only: false,
                    },
                    expiry: 0,
                    traffic_reset: std::borrow::Cow::Borrowed("never"),
                    last_traffic_reset_time: 0,
                    up_bytes: 1024 * 1024 * 500,
                    down_bytes: 1024 * 1024 * 1500,
                    total_limit: 0,
                    all_time: 0,
                    extra: std::collections::HashMap::new(),
                    up_speed_limit: 0,
                    down_speed_limit: 0,
                }),
                server_address: "example.com".to_string(),
            }
        }
    }
}
