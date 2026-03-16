//! Forms Demo Page
//!
//! Demonstration page showcasing all form components.

use crate::ui::components::card::Card;
use crate::ui::components::forms::*;
use dioxus::prelude::*;

#[component]
pub fn FormsDemoPage() -> Element {
    // State for all form components
    let mut text_value = use_signal(|| String::new());
    let mut number_value = use_signal(|| 0i64);
    let mut switch_value = use_signal(|| false);
    let mut textarea_value = use_signal(|| String::new());
    let mut multi_text_value = use_signal(|| vec![String::from("example.com")]);
    let mut date_value = use_signal(|| None::<i64>);
    let mut choice_value = use_signal(|| None::<String>);

    // Protocol options for ChoiceBox
    let protocol_options = vec![
        ChoiceBoxOption {
            value: "vless".to_string(),
            label: "VLESS".to_string(),
            description: Some("Lightweight protocol with XTLS support".to_string()),
            icon: None,
        },
        ChoiceBoxOption {
            value: "vmess".to_string(),
            label: "VMess".to_string(),
            description: Some("Original V2Ray protocol".to_string()),
            icon: None,
        },
        ChoiceBoxOption {
            value: "trojan".to_string(),
            label: "Trojan".to_string(),
            description: Some("TLS-based protocol".to_string()),
            icon: None,
        },
        ChoiceBoxOption {
            value: "shadowsocks".to_string(),
            label: "Shadowsocks".to_string(),
            description: Some("Fast SOCKS5 proxy".to_string()),
            icon: None,
        },
    ];

    rsx! {
        div { class: "p-6 space-y-6",
            h1 { class: "text-2xl font-bold text-white mb-6", "Form Components Demo" }

            div { class: "grid grid-cols-1 lg:grid-cols-2 gap-6",
                // Text Input Demo
                Card {
                    title: "TextInput".to_string(),
                    div { class: "space-y-4",
                        TextInput {
                            label: Some("Username".to_string()),
                            value: text_value,
                            placeholder: Some("Enter username...".to_string()),
                            description: Some("Your unique username".to_string()),
                            prefix_icon: Some("person".to_string()),
                            required: true,
                        }

                        div { class: "text-xs text-gray-500",
                            "Current value: \"{text_value()}\""
                        }
                    }
                }

                // Number Input Demo
                Card {
                    title: "NumberInput".to_string(),
                    div { class: "space-y-4",
                        NumberInput {
                            label: Some("Port".to_string()),
                            value: number_value,
                            min: Some(1024),
                            max: Some(65535),
                            step: 1,
                            description: Some("Server port (1024-65535)".to_string()),
                            required: true,
                        }

                        div { class: "text-xs text-gray-500",
                            "Current value: {number_value()}"
                        }
                    }
                }

                // Switch Demo
                Card {
                    title: "Switch".to_string(),
                    div { class: "space-y-4",
                        Switch {
                            label: Some("Enable Feature".to_string()),
                            value: switch_value,
                        }

                        div { class: "text-xs text-gray-500",
                            "Current value: {switch_value()}"
                        }
                    }
                }

                // ChoiceBox Demo
                Card {
                    title: "ChoiceBox".to_string(),
                    div { class: "space-y-4",
                        ChoiceBox {
                            label: Some("Protocol".to_string()),
                            value: choice_value,
                            options: protocol_options,
                            placeholder: "Select a protocol...".to_string(),
                            description: Some("Choose your preferred protocol".to_string()),
                            required: true,
                        }

                        div { class: "text-xs text-gray-500",
                            "Selected: {choice_value().unwrap_or_else(|| \"None\".to_string())}"
                        }
                    }
                }

                // TextArea Demo
                Card {
                    title: "TextArea".to_string(),
                    div { class: "space-y-4",
                        TextArea {
                            label: Some("Description".to_string()),
                            value: textarea_value,
                            placeholder: Some("Enter description...".to_string()),
                            rows: 4,
                            show_count: true,
                            max_length: Some(500),
                            show_copy: true,
                        }

                        div { class: "text-xs text-gray-500",
                            "Length: {textarea_value().len()} chars"
                        }
                    }
                }

                // MultiTextInput Demo
                Card {
                    title: "MultiTextInput".to_string(),
                    div { class: "space-y-4",
                        MultiTextInput {
                            label: Some("SNI List".to_string()),
                            value: multi_text_value,
                            placeholder: "Enter domain...".to_string(),
                            description: Some("Server Name Indication domains".to_string()),
                            min_entries: 1,
                            max_entries: Some(10),
                        }

                        div { class: "text-xs text-gray-500",
                            "Entries: {multi_text_value().len()}"
                        }
                    }
                }

                // DatePicker Demo
                Card {
                    title: "DatePicker".to_string(),
                    div { class: "space-y-4",
                        DatePicker {
                            label: Some("Expiry Date".to_string()),
                            value: date_value,
                            description: Some("Account expiration date".to_string()),
                            include_time: true,
                        }

                        div { class: "text-xs text-gray-500",
                            "Selected: {date_value().map(|v| v.to_string()).unwrap_or_else(|| \"None\".to_string())}"
                        }
                    }
                }
            }
        }
    }
}
