//! Cloudflare/IP Scanner Orchestrator
//!
//! Manages the autonomous discovery of clean paths.

use crate::domain::models::{CleanPath, ScannerConfig};
use crate::ui::components::card::Card;
use crate::ui::components::forms::{NumberInput, TextArea};
use crate::ui::server_fns::{get_scanner_results, trigger_scanner_pulse};
use dioxus::prelude::*;

#[component]
pub fn CfScannerPage() -> Element {
    let mut concurrency = use_signal(|| 50i64);
    let mut timeout = use_signal(|| 1000i64);
    let mut cidrs = use_signal(|| "104.16.0.0/12\n172.64.0.0/13".to_string());

    let mut logs = use_signal(|| Vec::<String>::new());
    let mut paths = use_signal(|| Vec::<CleanPath>::new());
    let mut scanning = use_signal(|| false);

    // Initial load
    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        if let Ok(res) = get_scanner_results().await {
            paths.set(res);
        }
    });

    let handle_scan = move |_| async move {
        scanning.set(true);
        logs.write()
            .push(format!("Starting scan with {} threads...", concurrency()));

        let config = ScannerConfig {
            concurrency: *concurrency.read() as u32,
            timeout_ms: *timeout.read() as u32,
            cidr_ranges: cidrs.read().lines().map(String::from).collect(),
        };

        if trigger_scanner_pulse(config).await.is_ok() {
            // Mock streaming logs
            logs.write().push("Scanner initialized.".to_string());
            logs.write().push("Loaded 2 CIDR ranges.".to_string());

            // In real app, we'd stream these or poll.
            // Mocking a successful find:
            if let Ok(res) = get_scanner_results().await {
                for path in &res {
                    logs.write().push(format!(
                        "Found Clean IP: [{}] | ISP: [{}] | Score: [{}]",
                        path.ip, path.isp, path.score
                    ));
                }
                paths.set(res);
            }
        }

        scanning.set(false);
    };

    rsx! {
        div { class: "p-6 space-y-6 animate-fade-in",
            h1 { class: "text-2xl font-bold bg-gradient-to-r from-text-main to-text-secondary bg-clip-text text-transparent",
                "Scanner Control Center"
            }

            div { class: "grid grid-cols-1 lg:grid-cols-3 gap-6",
                // Controls
                div { class: "lg:col-span-1 space-y-6",
                    Card {
                        title: "Orchestration".to_string(),
                        div { class: "p-4 space-y-4",
                            NumberInput {
                                label: Some("Concurrency".to_string()),
                                value: concurrency,
                                min: Some(1),
                                max: Some(1000),
                            }
                            NumberInput {
                                label: Some("Timeout (ms)".to_string()),
                                value: timeout,
                                min: Some(100),
                                max: Some(5000),
                            }
                            TextArea {
                                label: Some("Target CIDRs".to_string()),
                                value: cidrs,
                                rows: 5,
                            }

                            button {
                                class: "w-full py-2 px-4 bg-primary text-white rounded font-medium disabled:opacity-50 transition-colors hover:bg-primary/90",
                                onclick: handle_scan,
                                disabled: scanning,
                                if scanning() { "Scanning..." } else { "Launch Scan" }
                            }
                        }
                    }
                }

                // Log View
                div { class: "lg:col-span-2",
                    Card {
                        title: "Real-time Discovery Log".to_string(),
                        div { class: "p-4 h-[400px] bg-black/50 rounded overflow-y-auto font-mono text-xs text-green-400 p-2 space-y-1",
                            for log in logs() {
                                div { "{log}" }
                            }
                            if logs().is_empty() {
                                div { class: "text-gray-600 italic", "Waiting for command..." }
                            }
                        }
                    }
                }
            }

            // Clean Paths Table
            Card {
                title: "IP Aging Management".to_string(),
                div { class: "overflow-x-auto",
                    table { class: "w-full text-left",
                        thead { class: "bg-white/5 text-gray-400 text-xs uppercase",
                            tr {
                                th { class: "px-4 py-3", "IP Address" }
                                th { class: "px-4 py-3", "ISP" }
                                th { class: "px-4 py-3", "Score" }
                                th { class: "px-4 py-3", "Status" }
                                th { class: "px-4 py-3 text-right", "Actions" }
                            }
                        }
                        tbody { class: "divide-y divide-white/5 text-sm",
                            for path in paths() {
                                tr { class: "hover:bg-white/5 transition-colors",
                                    td { class: "px-4 py-3 font-mono", "{path.ip}" }
                                    td { class: "px-4 py-3", "{path.isp}" }
                                    td { class: "px-4 py-3 text-green-400 font-bold", "{path.score}" }
                                    td { class: "px-4 py-3",
                                        span { class: "px-2 py-1 rounded-full text-xs bg-green-500/10 text-green-400 border border-green-500/20",
                                            "{path.status}"
                                        }
                                    }
                                    td { class: "px-4 py-3 text-right space-x-2",
                                        button { class: "text-blue-400 hover:text-blue-300 text-xs uppercase font-medium", "Re-scan" }
                                        button { class: "text-red-400 hover:text-red-300 text-xs uppercase font-medium", "Blacklist" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
