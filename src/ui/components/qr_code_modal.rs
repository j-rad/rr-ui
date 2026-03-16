//! QR Code Modal Component
//!
//! Modal for displaying connection QR codes with technical inspection.

use crate::ui::components::forms::{ChoiceBoxOption, TextArea};
use crate::ui::components::modal::Modal;
use crate::ui::server_fns::generate_qr_code;
use base64::{Engine as _, engine::general_purpose};
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct QrCodeModalProps {
    /// Whether the modal is open
    pub open: Signal<bool>,
    /// Connection URL (Standard)
    pub connection_url: String,
    /// Full JSON Config (Optional)
    pub json_config: Option<String>,
    /// Connection name/remark
    pub remark: String,
    /// Called when modal is closed
    #[props(default)]
    pub on_close: Option<EventHandler<()>>,
}

#[derive(Clone, PartialEq)]
enum QrMode {
    Standard,
    Json,
    Compressed,
}

#[component]
pub fn QrCodeModal(props: QrCodeModalProps) -> Element {
    let mut mode = use_signal(|| QrMode::Standard);

    // Derived payload based on mode
    let payload = use_memo(move || {
        match mode() {
            QrMode::Standard => props.connection_url.clone(),
            QrMode::Json => props
                .json_config
                .clone()
                .unwrap_or_else(|| props.connection_url.clone()),
            QrMode::Compressed => {
                // Placeholder for compression logic (e.g. deflate + base64)
                // For MVP, just wrap in base64
                let content = props
                    .json_config
                    .clone()
                    .unwrap_or_else(|| props.connection_url.clone());
                general_purpose::STANDARD.encode(content)
            }
        }
    });

    rsx! {
        Modal {
            open: props.open,
            title: format!("Share - {}", props.remark),
            width: "500px".to_string(),
            on_close: props.on_close.clone(),

            div { class: "flex flex-col gap-6",
                // Mode Selector
                div { class: "flex bg-bg-tertiary p-1 rounded-lg gap-1",
                    button {
                        class: if mode() == QrMode::Standard {
                            "flex-1 py-1.5 text-xs font-medium text-white bg-primary rounded shadow-sm transition-all"
                        } else {
                            "flex-1 py-1.5 text-xs font-medium text-gray-400 hover:text-white transition-all"
                        },
                        onclick: move |_| mode.set(QrMode::Standard),
                        "Standard URI"
                    }
                    button {
                        class: if mode() == QrMode::Json {
                            "flex-1 py-1.5 text-xs font-medium text-white bg-primary rounded shadow-sm transition-all"
                        } else {
                            "flex-1 py-1.5 text-xs font-medium text-gray-400 hover:text-white transition-all"
                        },
                        onclick: move |_| mode.set(QrMode::Json),
                        "Full JSON"
                    }
                    button {
                        class: if mode() == QrMode::Compressed {
                            "flex-1 py-1.5 text-xs font-medium text-white bg-primary rounded shadow-sm transition-all"
                        } else {
                            "flex-1 py-1.5 text-xs font-medium text-gray-400 hover:text-white transition-all"
                        },
                        onclick: move |_| mode.set(QrMode::Compressed),
                        "Compressed"
                    }
                }

                // QR Code Display
                div { class: "flex justify-center",
                    div { class: "w-64 h-64 bg-white p-2 rounded-lg shadow-lg overflow-hidden",
                        {
                            let current_payload = payload();
                            let qr_svg = use_resource(move || {
                                let p = current_payload.clone();
                                async move {
                                    generate_qr_code(p).await
                                }
                            });

                            match &*qr_svg.read_unchecked() {
                                Some(Ok(svg_content)) => rsx! {
                                    div {
                                        class: "w-full h-full",
                                        dangerous_inner_html: "{svg_content}"
                                    }
                                },
                                Some(Err(e)) => rsx! {
                                    div { class: "flex items-center justify-center h-full text-red-500 text-xs text-center p-4",
                                        "Error: {e}"
                                    }
                                },
                                None => rsx! {
                                    div { class: "flex items-center justify-center h-full text-gray-400 text-xs animate-pulse",
                                        "Generating..."
                                    }
                                }
                            }
                        }
                    }
                }

                // Config Inspector
                div { class: "space-y-2",
                    div { class: "flex justify-between items-center",
                        label { class: "text-xs font-medium text-gray-400 uppercase tracking-wider", "Payload Inspector" }
                        span { class: "text-xs text-gray-500 font-mono", "{payload().len()} bytes" }
                    }
                    TextArea {
                        label: None,
                        value: use_signal(move || payload()),
                        rows: 4,
                        monospace: true,
                        show_copy: true,
                    }
                }
            }
        }
    }
}
