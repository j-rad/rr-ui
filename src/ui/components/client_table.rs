//! Client Table Component
//!
//! Nested table for displaying clients within an inbound.

use crate::ui::components::forms::Switch;
use crate::ui::components::qr_code_modal::QrCodeModal;
use dioxus::prelude::*;
use serde_json::json;

#[derive(Clone, PartialEq)]
pub struct ClientData {
    pub email: String,
    pub uuid: String,
    pub enabled: bool,
    pub total_gb: Option<i64>,
    pub expiry_time: Option<i64>,
    pub up: i64,
    pub down: i64,
    /// Full share URL (VLESS / VMess / Flow-J) for this client.
    /// When set, the Share button will open the QR modal with this payload.
    pub connection_url: Option<String>,
    /// Full JSON Config for advanced clients (Multiport/MQTT)
    pub json_config: Option<String>,
}

#[derive(Props, Clone, PartialEq)]
pub struct ClientTableProps {
    pub clients: Vec<ClientData>,
    pub inbound_id: i64,
    #[props(default)]
    pub on_edit: Option<EventHandler<String>>,
    #[props(default)]
    pub on_delete: Option<EventHandler<String>>,
    #[props(default)]
    pub on_qr: Option<EventHandler<String>>,
    #[props(default)]
    pub on_manage: Option<EventHandler<String>>,
    #[props(default)]
    pub on_share: Option<EventHandler<String>>,
}

#[component]
pub fn ClientTable(props: ClientTableProps) -> Element {
    // Per-table signals for the Share / QR modal
    let mut share_open = use_signal(|| false);
    let mut share_url = use_signal(|| String::new());
    let mut share_json = use_signal(|| None::<String>);
    let mut share_remark = use_signal(|| String::new());

    rsx! {
        div { class: "bg-[#0f0f0f] border-t border-border",
            // Table Header
            div { class: "grid grid-cols-12 gap-4 px-6 py-2 text-xs font-semibold text-gray-500 uppercase tracking-wider bg-[#1a1a1a]",
                div { class: "col-span-3", "Client" }
                div { class: "col-span-2 text-center", "Status" }
                div { class: "col-span-3 text-center", "Traffic" }
                div { class: "col-span-2 text-center", "Expiry" }
                div { class: "col-span-2 text-center", "Actions" }
            }

            // Client Rows
            div { class: "divide-y divide-border",
                for client in &props.clients {
                    {
                        let email = client.email.clone();
                        let uuid = client.uuid.clone();
                        let enabled = client.enabled;
                        let client_url = client.connection_url.clone();
                        let client_json = client.json_config.clone();
                        rsx! {
                            div {
                                key: "{email}",
                                class: "grid grid-cols-12 gap-4 px-6 py-3 items-center hover:bg-[#1a1a1a] transition-colors text-sm",

                                // Client Email
                                div { class: "col-span-3",
                                    div { class: "flex items-center gap-2",
                                        span {
                                            class: if enabled { "w-2 h-2 rounded-full bg-green-500" } else { "w-2 h-2 rounded-full bg-gray-500" },
                                        }
                                        span { class: "text-gray-300 font-medium truncate", "{email}" }
                                    }
                                }

                                // Status
                                div { class: "col-span-2 flex justify-center",
                                    Switch {
                                        value: use_signal(|| enabled),
                                        label: None,
                                    }
                                }

                                // Traffic
                                div { class: "col-span-3 flex justify-center",
                                    div { class: "flex items-center gap-2",
                                        span { class: "text-gray-400 text-xs",
                                            "{format_bytes(client.up + client.down)}"
                                        }
                                        if let Some(total) = client.total_gb {
                                            span { class: "text-gray-600 text-xs", "/ {total}GB" }
                                        } else {
                                            span { class: "text-gray-600 text-xs", "/ ∞" }
                                        }
                                    }
                                }

                                // Expiry
                                div { class: "col-span-2 flex justify-center",
                                    if let Some(expiry) = client.expiry_time {
                                        span { class: "text-gray-400 text-xs", "{format_timestamp(expiry)}" }
                                    } else {
                                        span { class: "text-gray-600 text-xs", "∞" }
                                    }
                                }

                                // Actions
                                div { class: "col-span-2 flex justify-center gap-2",
                                    {
                                        let email_qr = email.clone();
                                        let email_edit = email.clone();
                                        let email_manage = email.clone();
                                        let email_delete = email.clone();
                                        let email_share = email.clone();
                                        let uuid_share = uuid.clone();
                                        let url_for_share = client_url.clone();
                                        let json_for_share = client_json.clone();
                                        rsx! {
                                            // ── Share / QR button ──────────────────────────────
                                            button {
                                                class: "p-1 text-gray-500 hover:text-emerald-400 transition-colors",
                                                title: "Share Config",
                                                onclick: move |_| {
                                                    // Open inline QR modal with full connection URL
                                                    if let Some(url) = &url_for_share {
                                                        share_url.set(url.clone());
                                                    } else {
                                                        share_url.set(String::new());
                                                    }

                                                    if let Some(json) = &json_for_share {
                                                        share_json.set(Some(json.clone()));
                                                    } else {
                                                        // Generate default Multiport/MQTT config
                                                        share_json.set(Some(generate_default_config(&email_share, &uuid_share)));
                                                    }

                                                    share_remark.set(email_share.clone());
                                                    share_open.set(true);

                                                    // Also propagate to parent if interested
                                                    if let Some(ref handler) = props.on_share {
                                                        handler.call(email_share.clone());
                                                    }
                                                },
                                                span { class: "material-symbols-outlined text-[18px]", "share" }
                                            }
                                            button {
                                                class: "p-1 text-gray-500 hover:text-primary transition-colors",
                                                title: "QR Code",
                                                onclick: move |_| {
                                                    if let Some(ref handler) = props.on_qr {
                                                        handler.call(email_qr.clone());
                                                    }
                                                },
                                                span { class: "material-symbols-outlined text-[18px]", "qr_code_2" }
                                            }
                                            button {
                                                class: "p-1 text-gray-500 hover:text-blue-400 transition-colors",
                                                title: "Edit",
                                                onclick: move |_| {
                                                    if let Some(ref handler) = props.on_edit {
                                                        handler.call(email_edit.clone());
                                                    }
                                                },
                                                span { class: "material-symbols-outlined text-[18px]", "edit" }
                                            }
                                            button {
                                                class: "p-1 text-gray-500 hover:text-purple-400 transition-colors",
                                                title: "Details",
                                                onclick: move |_| {
                                                    if let Some(ref handler) = props.on_manage {
                                                        handler.call(email_manage.clone());
                                                    }
                                                },
                                                span { class: "material-symbols-outlined text-[18px]", "settings" }
                                            }
                                            button {
                                                class: "p-1 text-gray-500 hover:text-red-500 transition-colors",
                                                title: "Delete",
                                                onclick: move |_| {
                                                    if let Some(ref handler) = props.on_delete {
                                                        handler.call(email_delete.clone());
                                                    }
                                                },
                                                span { class: "material-symbols-outlined text-[18px]", "delete" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Shared QR Code Modal (one per table, shown on Share click)
            QrCodeModal {
                open: share_open,
                connection_url: share_url(),
                json_config: share_json(),
                remark: share_remark(),
                on_close: move |_| share_open.set(false),
            }
        }
    }
}

fn format_bytes(bytes: i64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

fn format_timestamp(ts: i64) -> String {
    // Simplified - in production use chrono
    format!("{}d", ts / 86400)
}

fn generate_default_config(email: &str, uuid: &str) -> String {
    json!({
        "v": "2",
        "ps": email,
        "add": "auto",
        "port": "443",
        "id": uuid,
        "net": "tcp",
        "type": "none",
        "tls": "tls",
        "multiport": {
            "enabled": true,
            "pool_size": 8,
            "rotation_min": 60
        },
        "mqtt": {
            "enabled": true,
            "topic": format!("rr/{}", uuid)
        }
    }).to_string()
}
