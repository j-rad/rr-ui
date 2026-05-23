//! Logs Page

use crate::ui::components::log_viewer::{LogEntry, LogViewer};
use crate::ui::server_fns::get_audit_logs;
use dioxus::prelude::*;

#[component]
pub fn LogsPage() -> Element {
    // Fetch logs using resource
    let mut logs_resource = use_resource(move || async move {
        match get_audit_logs(100).await {
            Ok(logs) => logs,
            Err(e) => {
                log::error!("Failed to fetch audit logs: {}", e);
                vec![]
            }
        }
    });

    let system_logs = match &*logs_resource.read_unchecked() {
        Some(logs) => logs
            .iter()
            .map(|event| LogEntry {
                timestamp: chrono::DateTime::from_timestamp(event.timestamp / 1000, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "Unknown".to_string()),
                level: if event.success {
                    "INFO".to_string()
                } else {
                    "ERROR".to_string()
                },
                message: format!("{:?}", event.action),
                details: format!(
                    "User: {}, IP: {}",
                    event.user.as_deref().unwrap_or("Unknown"),
                    event.ip_address.as_deref().unwrap_or("Unknown")
                ),
            })
            .collect(),
        None => vec![],
    };

    rsx! {
        div { class: "p-6 space-y-6",
            div { class: "flex justify-between items-center mb-6",
                h1 { class: "text-2xl font-bold text-white", "System Audit Logs" }
                button {
                    class: "px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition",
                    onclick: move |_| logs_resource.restart(),
                    "Refresh Logs"
                }
            }

            if logs_resource.read().is_none() {
                div { class: "text-center text-gray-400 py-12", "Loading logs..." }
            } else {
                div { class: "grid grid-cols-1 gap-6",
                    LogViewer {
                        logs: system_logs,
                        title: "Audit Trail".to_string(),
                    }
                }
            }
        }
    }
}
