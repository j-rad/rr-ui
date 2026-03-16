//! Log Viewer Component
//!
//! High-performance virtualized log viewer with filtering.

use crate::ui::components::forms::TextInput;
use dioxus::prelude::*;
use std::collections::VecDeque;

#[derive(Clone, PartialEq)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

#[derive(Props, Clone, PartialEq)]
pub struct LogViewerProps {
    // We accept raw logs, but internal state manages buffer
    pub logs: Vec<LogEntry>,
    #[props(default = "System Logs".to_string())]
    pub title: String,
}

#[component]
pub fn LogViewer(props: LogViewerProps) -> Element {
    let mut filter_regex = use_signal(|| String::new());
    let mut buffer_limit = use_signal(|| 10000usize);

    // Compute filtered logs directly
    let filtered_logs = {
        let regex_str = filter_regex.read();
        if regex_str.is_empty() {
            props.logs.clone()
        } else if let Ok(re) = regex::Regex::new(&regex_str) {
            props
                .logs
                .iter()
                .filter(|log| re.is_match(&log.message) || re.is_match(&log.level))
                .cloned()
                .collect::<Vec<_>>()
        } else {
            // Fallback to simple contains
            props
                .logs
                .iter()
                .filter(|log| log.message.contains(&*regex_str))
                .cloned()
                .collect::<Vec<_>>()
        }
    };

    rsx! {
        div { class: "bg-bg-panel border border-border rounded-lg overflow-hidden flex flex-col h-[500px]",
            // Header
            div { class: "px-4 py-3 border-b border-border flex items-center justify-between bg-bg-secondary",
                h3 { class: "text-sm font-semibold text-white", "{props.title}" }
                div { class: "flex items-center gap-2",
                    input {
                        class: "bg-black/30 border border-border rounded px-2 py-1 text-xs text-white w-48 focus:border-primary focus:outline-none",
                        placeholder: "Regex Filter...",
                        value: "{filter_regex}",
                        oninput: move |e| filter_regex.set(e.value()),
                    }
                    div { class: "text-xs text-gray-500 font-mono",
                        "{filtered_logs.len()} lines"
                    }
                }
            }

            // Log content (Virtualized container placeholder)
            // For true 10k+ virtualization in DOM, we'd need a specific Dioxus virtualization logic or careful rendering.
            // Here we render visible window or just list. Dioxus is fast, but 10k elements is heavy.
            // We'll limit render to last 500 filtered for UI performance, assuming "buffer" holds more.
            div { class: "flex-1 overflow-y-auto bg-black/30 font-mono text-xs p-2",
                for log in filtered_logs.iter().rev().take(500) { // Show latest 500
                    {
                        let level_color = match log.level.as_str() {
                            "ERROR" => "text-red-400",
                            "WARN" => "text-yellow-400",
                            "INFO" => "text-blue-400",
                            "DEBUG" => "text-gray-500",
                            _ => "text-gray-400",
                        };

                        rsx! {
                            div {
                                class: "flex gap-2 py-0.5 hover:bg-white/5 border-b border-white/5",
                                span { class: "text-gray-600 min-w-[140px]", "{log.timestamp}" }
                                span { class: "{level_color} font-bold min-w-[50px]", "{log.level}" }
                                span { class: "text-gray-300 break-all", "{log.message}" }
                            }
                        }
                    }
                }

                if filtered_logs.len() > 500 {
                    div { class: "text-center text-gray-500 py-2 italic",
                        "... {filtered_logs.len() - 500} older lines hidden (filter to see) ..."
                    }
                }

                if props.logs.is_empty() {
                    div { class: "flex items-center justify-center h-full text-gray-600",
                        "No logs available"
                    }
                }
            }
        }
    }
}
