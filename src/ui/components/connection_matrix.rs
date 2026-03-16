//! Connection Matrix Component
//!
//! Real-time table view of active connections (granular tracking).
//! Polls the backend every second.

use crate::ui::server_fns::get_active_connections;
use dioxus::prelude::*;
use std::time::SystemTime;

/// Format bytes to human readable string
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Get protocol badge styling (background, text, border classes)
/// Returns a tuple of (bg_class, text_class, border_class)
fn get_protocol_style(protocol: &str) -> (&'static str, &'static str, &'static str) {
    match protocol.to_lowercase().as_str() {
        "vless" => ("bg-cyan-500/20", "text-cyan-300", "border-cyan-500/30"),
        "vmess" => (
            "bg-purple-500/20",
            "text-purple-300",
            "border-purple-500/30",
        ),
        "trojan" => ("bg-amber-500/20", "text-amber-300", "border-amber-500/30"),
        "shadowsocks" | "ss" => ("bg-green-500/20", "text-green-300", "border-green-500/30"),
        "socks" | "socks5" => ("bg-blue-500/20", "text-blue-300", "border-blue-500/30"),
        "http" | "https" => (
            "bg-orange-500/20",
            "text-orange-300",
            "border-orange-500/30",
        ),
        "wireguard" | "wg" => ("bg-rose-500/20", "text-rose-300", "border-rose-500/30"),
        "reality" => (
            "bg-fuchsia-500/20",
            "text-fuchsia-300",
            "border-fuchsia-500/30",
        ),
        "tcp" => ("bg-slate-500/20", "text-slate-300", "border-slate-500/30"),
        "udp" => ("bg-teal-500/20", "text-teal-300", "border-teal-500/30"),
        // Fallback for unknown protocols
        _ => ("bg-gray-500/20", "text-gray-300", "border-gray-500/30"),
    }
}

/// Format duration (seconds) to HH:MM:SS
fn format_duration(start_time: u64) -> String {
    let now = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let duration = if now > start_time {
        now - start_time
    } else {
        0
    };

    let hours = duration / 3600;
    let minutes = (duration % 3600) / 60;
    let seconds = duration % 60;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}", minutes, seconds)
    }
}

#[component]
pub fn ConnectionMatrix() -> Element {
    let mut sessions = use_signal(Vec::new);
    let mut error_msg = use_signal(|| None::<String>);

    // Poll for active connections
    use_effect(move || {
        spawn(async move {
            loop {
                match get_active_connections().await {
                    Ok(data) => {
                        sessions.set(data);
                        error_msg.set(None);
                    }
                    Err(e) => {
                        let err: Option<String> = Some(e.to_string());
                        error_msg.set(err);
                    }
                }

                #[cfg(feature = "web")]
                crate::ui::sleep::sleep(1000 as u64).await;
                #[cfg(not(feature = "web"))]
                tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            }
        });
    });

    rsx! {
        div { class: "connection-matrix-container w-full h-full bg-white/5 backdrop-blur-md rounded-xl border border-white/10 overflow-hidden flex flex-col",
            // Header
            div { class: "p-4 border-b border-white/10 flex justify-between items-center bg-white/5",
                div { class: "flex items-center gap-2",
                    i { class: "fas fa-network-wired text-cyan-400" }
                    h3 { class: "font-semibold text-white", "Active Sessions" }
                    span { class: "bg-cyan-500/20 text-cyan-300 text-xs px-2 py-0.5 rounded-full border border-cyan-500/30",
                        "{sessions.read().len()}"
                    }
                }
                div { class: "text-xs text-gray-400 font-mono",
                    "Real-time (1s)"
                }
            }

            // Error Banner
            if let Some(err) = error_msg() {
                div { class: "bg-red-500/10 border-l-4 border-red-500 p-2 text-red-400 text-sm",
                    "Connection Error: {err}"
                }
            }

            // Table Container
            div { class: "flex-1 overflow-auto custom-scrollbar",
                table { class: "w-full text-left border-collapse",
                    thead { class: "bg-black/20 sticky top-0 backdrop-blur-sm z-10",
                        tr { class: "text-xs font-medium text-gray-400 uppercase tracking-wider",
                            th { class: "p-3 font-medium", "ID" }
                            th { class: "p-3 font-medium", "Protocol" }
                            th { class: "p-3 font-medium", "Source" }
                            th { class: "p-3 font-medium", "Destination" }
                            th { class: "p-3 font-medium text-right", "Upload" }
                            th { class: "p-3 font-medium text-right", "Download" }
                            th { class: "p-3 font-medium text-right", "Duration" }
                        }
                    }
                    tbody { class: "text-sm text-gray-300 divide-y divide-white/5",
                        if sessions.read().is_empty() {
                            tr {
                                td { colspan: "7", class: "p-8 text-center text-gray-500 italic",
                                    "No active connections"
                                }
                            }
                        }
                        for session in sessions.read().iter() {
                            {
                                let truncated_id = if session.id.len() > 8 { format!("{}...", &session.id[..8]) } else { session.id.clone() };
                                let (bg_class, text_class, border_class) = get_protocol_style(&session.protocol);
                                let badge_class = format!("px-2 py-1 rounded text-xs font-semibold {} {} border {}", bg_class, text_class, border_class);
                                rsx! {
                            tr { class: "hover:bg-white/5 transition-colors duration-150 group",
                                key: "{session.id}",
                                td { class: "p-3 font-mono text-xs text-gray-500 group-hover:text-gray-300",
                                    "{truncated_id}"
                                }
                                td { class: "p-3",
                                    span { class: "{badge_class}",
                                        "{session.protocol.to_uppercase()}"
                                    }
                                }
                                td { class: "p-3 font-mono text-xs text-gray-400", "{session.source}" }
                                td { class: "p-3 font-mono text-xs text-cyan-300", "{session.dest}" }
                                td { class: "p-3 text-right font-mono text-xs text-blue-300", "{format_bytes(session.uploaded)}" }
                                td { class: "p-3 text-right font-mono text-xs text-green-300", "{format_bytes(session.downloaded)}" }
                                td { class: "p-3 text-right font-mono text-xs text-gray-400", "{format_duration(session.start_time)}" }
                            }
                        }}
                        }
                    }
                }
            }
        }
    }
}
