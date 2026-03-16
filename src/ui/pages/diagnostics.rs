//! Diagnostics Page
//!
//! Tools for network diagnostics including Speed Test and censorship scanners.

use crate::domain::models::{ServerConfig, SpeedTestResults};
use crate::ui::components::card::Card;
use crate::ui::server_fns::trigger_speed_test;
use dioxus::prelude::*;

#[component]
pub fn DiagnosticsPage() -> Element {
    let mut speed_test_running = use_signal(|| false);
    let mut speed_test_results = use_signal(|| None::<SpeedTestResults>);
    let mut error_msg = use_signal(|| None::<String>);

    // Scanner State
    let mut selected_scanner = use_signal(|| crate::domain::models::ScannerType::Dns);
    let mut scanner_running = use_signal(|| false);
    let mut scan_results = use_signal(|| Vec::<crate::domain::models::ScanResult>::new());
    let mut scanner_error = use_signal(|| None::<String>);

    let run_speed_test = move |_| async move {
        speed_test_running.set(true);
        error_msg.set(None);
        speed_test_results.set(None);

        let config = ServerConfig {
            host: "speed.cloudflare.com".to_string(), // Default target
            port: 443,
            ..Default::default()
        };

        match trigger_speed_test(config).await {
            Ok(results) => {
                speed_test_results.set(Some(results));
            }
            Err(e) => {
                error_msg.set(Some(format!("{}", e)));
            }
        }
        speed_test_running.set(false);
    };

    let run_scan = move |_| async move {
        scanner_running.set(true);
        scanner_error.set(None);
        scan_results.set(Vec::new());

        let scanner_type = selected_scanner();

        match crate::ui::server_fns::run_scanner(scanner_type).await {
            Ok(results) => {
                scan_results.set(results);
            }
            Err(e) => {
                scanner_error.set(Some(format!("{}", e)));
            }
        }
        scanner_running.set(false);
    };

    rsx! {
        div { class: "p-6 space-y-6 animate-fade-in",
            div { class: "flex items-center justify-between mb-6",
                div {
                    h1 { class: "text-2xl font-bold bg-gradient-to-r from-text-main to-text-secondary bg-clip-text text-transparent tracking-tight", "Diagnostics" }
                    p { class: "text-sm text-text-muted mt-1", "Network analysis and troubleshooting tools" }
                }
            }

            // Speed Test Section
            Card {
                title: "Network Speed Test".to_string(),
                div { class: "p-4 space-y-4",
                    div { class: "flex items-center gap-4",
                        button {
                            class: "px-4 py-2 bg-primary text-white rounded-lg hover:bg-primary/90 transition-colors disabled:opacity-50 flex items-center gap-2",
                            onclick: run_speed_test,
                            disabled: speed_test_running,
                            if speed_test_running() {
                                span { class: "material-symbols-outlined animate-spin", "sync" }
                                "Testing..."
                            } else {
                                span { class: "material-symbols-outlined", "speed" }
                                "Start Speed Test"
                            }
                        }
                    }

                    if let Some(error) = error_msg() {
                        div { class: "p-4 bg-red-500/10 border border-red-500/20 rounded-lg text-red-500 flex items-center gap-2",
                            span { class: "material-symbols-outlined", "error" }
                            "{error}"
                        }
                    }

                    if let Some(results) = speed_test_results() {
                        div { class: "grid grid-cols-2 md:grid-cols-4 gap-4 mt-6",
                            div { class: "p-4 bg-glass-bg/30 rounded-lg border border-glass-border text-center",
                                div { class: "text-gray-400 text-xs uppercase tracking-wider mb-1", "Latency" }
                                div { class: "text-2xl font-bold text-text-main", "{results.latency_ms:.1} ms" }
                            }
                            div { class: "p-4 bg-glass-bg/30 rounded-lg border border-glass-border text-center",
                                div { class: "text-gray-400 text-xs uppercase tracking-wider mb-1", "Jitter" }
                                div { class: "text-2xl font-bold text-text-main", "{results.jitter_ms:.1} ms" }
                            }
                            div { class: "p-4 bg-glass-bg/30 rounded-lg border border-glass-border text-center",
                                div { class: "text-gray-400 text-xs uppercase tracking-wider mb-1", "Download" }
                                div { class: "text-2xl font-bold text-green-400", "{results.download_mbps:.1} Mbps" }
                            }
                            div { class: "p-4 bg-glass-bg/30 rounded-lg border border-glass-border text-center",
                                div { class: "text-gray-400 text-xs uppercase tracking-wider mb-1", "Packet Loss" }
                                div { class: "text-2xl font-bold text-text-main", "{results.packet_loss:.1}%" }
                            }
                        }
                    }
                }
            }

            // Scanner Section
            Card {
                title: "Network Scanner".to_string(),
                div { class: "p-4 space-y-6",
                    // Type Selector
                    div { class: "flex gap-2",
                         button {
                             class: if selected_scanner() == crate::domain::models::ScannerType::Dns {
                                 "px-4 py-2 text-sm font-medium rounded-lg transition-colors bg-primary text-white"
                             } else {
                                 "px-4 py-2 text-sm font-medium rounded-lg transition-colors hover:bg-white/5 text-gray-400"
                             },
                             onclick: move |_| selected_scanner.set(crate::domain::models::ScannerType::Dns),
                             "DNS Scanner"
                         }
                         button {
                             class: if selected_scanner() == crate::domain::models::ScannerType::Cloudflare {
                                 "px-4 py-2 text-sm font-medium rounded-lg transition-colors bg-primary text-white"
                             } else {
                                 "px-4 py-2 text-sm font-medium rounded-lg transition-colors hover:bg-white/5 text-gray-400"
                             },
                             onclick: move |_| selected_scanner.set(crate::domain::models::ScannerType::Cloudflare),
                             "CDNs Scanner"
                         }
                    }

                    div { class: "flex items-center gap-4",
                         button {
                            class: "px-4 py-2 bg-primary text-white rounded-lg hover:bg-primary/90 transition-colors disabled:opacity-50 flex items-center gap-2",
                            onclick: run_scan,
                            disabled: scanner_running,
                            if scanner_running() {
                                span { class: "material-symbols-outlined animate-spin", "sync" }
                                "Scanning..."
                            } else {
                                span { class: "material-symbols-outlined", "radar" }
                                "Scan Network"
                            }
                        }
                    }

                    if let Some(error) = scanner_error() {
                        div { class: "p-4 bg-red-500/10 border border-red-500/20 rounded-lg text-red-500 flex items-center gap-2",
                            span { class: "material-symbols-outlined", "error" }
                            "{error}"
                        }
                    }

                    if !scan_results().is_empty() {
                        div { class: "border border-border rounded-lg overflow-hidden",
                            div { class: "bg-surface-dark px-4 py-2 text-xs font-medium text-gray-400 uppercase tracking-wider grid grid-cols-4",
                                div { "IP Address" }
                                div { "Status" }
                                div { "Latency" }
                                div { "Resolver Type" }
                            }
                             div { class: "divide-y divide-border",
                                 for result in scan_results() {
                                     div { class: "px-4 py-3 text-sm text-gray-300 grid grid-cols-4 items-center hover:bg-white/5",
                                        div { "{result.ip}" }
                                        div { class: if result.status == "Clean" || result.status == "Accessible" { "text-emerald-400" } else { "text-red-400" }, "{result.status}" }
                                        div { "{result.latency_ms:.1} ms" }
                                        div { "{result.resolver_type.clone().unwrap_or_else(|| \"-\".to_string())}" }
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
