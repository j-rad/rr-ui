//! Client Table Component
//!
//! Nested table for displaying clients within an inbound.
//! Optimized with memoization and direct signal access for tactical performance.

use crate::ui::components::qr_code_modal::QrCodeModal;
use dioxus::prelude::*;
use serde_json::json;
use crate::domain::models::Client;

#[derive(Props, Clone, PartialEq)]
pub struct ClientTableProps {
    pub clients: Vec<Client>,
    pub inbound_id: String,
    #[props(default)]
    pub on_edit: Option<EventHandler<Client>>,
    #[props(default)]
    pub on_delete: Option<EventHandler<String>>,
    #[props(default)]
    pub on_qr: Option<EventHandler<String>>,
    #[props(default)]
    pub on_manage: Option<EventHandler<String>>,
    #[props(default)]
    pub on_share: Option<EventHandler<String>>,
}

#[derive(Props, Clone, PartialEq)]
pub struct ClientRowProps {
    pub client: Client,
    pub is_active: bool,
    pub on_edit: EventHandler<Client>,
    pub on_delete: EventHandler<String>,
    pub on_share: EventHandler<Client>,
}

/// Memoized individual client row to prevent unnecessary re-renders during high-frequency telemetry.
#[component]
pub fn ClientRow(props: ClientRowProps) -> Element {
    let row = use_memo(move || {
        let client = props.client.clone();
        let email = client.email.clone().unwrap_or_default();
        let status_color = if props.is_active {
            "bg-green-500 shadow-[0_0_8px_rgba(34,197,94,0.4)]"
        } else {
            "bg-slate-500"
        };

        let share_client = client.clone();
        let edit_client = client.clone();
        let delete_email = email.clone();

        rsx! {
            tr { class: "group hover:bg-white/[0.02] transition-colors border-b border-white/[0.05]",
                td { class: "py-4 px-6",
                    div { class: "flex items-center gap-3",
                        div { class: "w-2 h-2 rounded-full {status_color}" }
                        div {
                            div { class: "text-sm font-medium text-text-main", "{email}" }
                            div { class: "text-[10px] text-text-muted font-mono uppercase tracking-wider",
                                "{client.id.clone().unwrap_or_default()}"
                            }
                        }
                    }
                }
                td { class: "py-4 px-6",
                    span { class: "px-2 py-0.5 rounded text-[10px] font-bold bg-primary/10 text-primary border border-primary/20 uppercase tracking-tighter",
                        "Level {client.level.unwrap_or(0)}"
                    }
                }
                td { class: "py-4 px-6 font-mono text-xs text-text-muted",
                    "{format_bytes(client.up + client.down)}"
                }
                td { class: "py-4 px-6 text-right",
                    div { class: "flex justify-end gap-2 opacity-0 group-hover:opacity-100 transition-opacity",
                        button {
                            class: "p-2 hover:bg-white/10 rounded-lg text-text-muted hover:text-emerald-400 transition-colors",
                            onclick: move |_| props.on_share.call(share_client.clone()),
                            span { class: "material-symbols-outlined text-[18px]", "share" }
                        }
                        button {
                            class: "p-2 hover:bg-white/10 rounded-lg text-text-muted hover:text-primary transition-colors",
                            onclick: move |_| props.on_edit.call(edit_client.clone()),
                            span { class: "material-symbols-outlined text-[18px]", "edit" }
                        }
                        button {
                            class: "p-2 hover:bg-white/10 rounded-lg text-text-muted hover:text-red-400 transition-colors",
                            onclick: move |_| props.on_delete.call(delete_email.clone()),
                            span { class: "material-symbols-outlined text-[18px]", "delete" }
                        }
                    }
                }
            }
        }
    });

    row()
}

#[component]
pub fn ClientTable(props: ClientTableProps) -> Element {
    let mut share_open = use_signal(|| false);
    let mut share_client = use_signal(|| None::<Client>);

    let handle_edit = move |client: Client| {
        if let Some(ref handler) = props.on_edit {
            handler.call(client);
        }
    };

    let handle_delete = move |email: String| {
        if let Some(ref handler) = props.on_delete {
            handler.call(email);
        }
    };

    let handle_share = move |client: Client| {
        share_client.set(Some(client));
        share_open.set(true);
    };

    let share_url = move || {
        if let Some(client) = share_client.read().as_ref() {
            // Simplified sharing logic
            format!("vless://{}@node.edgeray.io:443?encryption=none&security=reality", 
                client.id.clone().unwrap_or_default())
        } else {
            String::new()
        }
    };

    let share_json = move || {
        if let Some(client) = share_client.read().as_ref() {
            Some(json!({
                "v": "2",
                "ps": client.email,
                "id": client.id,
                "multiport": { "enabled": true, "pool_size": 8 },
                "mqtt": { "enabled": true, "topic": format!("rr/{}", client.id.clone().unwrap_or_default()) }
            }).to_string())
        } else {
            None
        }
    };

    rsx! {
        div { class: "overflow-hidden rounded-xl border border-white/[0.05] bg-black/20 backdrop-blur-md",
            table { class: "w-full text-left border-collapse",
                thead { class: "bg-white/[0.02] border-b border-white/[0.05]",
                    tr {
                        th { class: "py-4 px-6 text-[11px] font-bold text-text-muted uppercase tracking-widest",
                            "Client / User ID"
                        }
                        th { class: "py-4 px-6 text-[11px] font-bold text-text-muted uppercase tracking-widest",
                            "Access Level"
                        }
                        th { class: "py-4 px-6 text-[11px] font-bold text-text-muted uppercase tracking-widest",
                            "Traffic Usage"
                        }
                        th { class: "py-4 px-6 text-right text-[11px] font-bold text-text-muted uppercase tracking-widest",
                            "Actions"
                        }
                    }
                }
                tbody {
                    if props.clients.is_empty() {
                        tr {
                            td { class: "py-12 text-center text-text-muted text-sm", colspan: 4,
                                "No clients configured for this inbound."
                            }
                        }
                    } else {
                        for client in props.clients.iter().cloned() {
                            ClientRow {
                                key: "{client.email.clone().unwrap_or_default()}",
                                client: client,
                                is_active: true, // Mocked for now
                                on_edit: handle_edit,
                                on_delete: handle_delete,
                                on_share: handle_share,
                            }
                        }
                    }
                }
            }
        }

        if *share_open.read() {
            QrCodeModal {
                open: share_open,
                connection_url: share_url(),
                json_config: share_json(),
                remark: share_client.read().as_ref().and_then(|c| c.email.clone()).unwrap_or_default(),
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
