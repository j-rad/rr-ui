use crate::ui::components::forms::{ChoiceBox, ChoiceBoxOption, TextInput, NumberInput};
use dioxus::prelude::*;

#[component]
pub fn SlipstreamForm(
    root_domain: Signal<String>,
    record_type: Signal<Option<String>>,
    udp_frag_limit: Signal<i64>,
) -> Element {
    let record_options = vec![
        ChoiceBoxOption {
            label: "TXT".to_string(),
            value: "txt".to_string(),
            description: None,
            icon: Some("description".to_string()),
        },
        ChoiceBoxOption {
            label: "A (IPv4)".to_string(),
            value: "a".to_string(),
            description: None,
            icon: Some("dns".to_string()),
        },
        ChoiceBoxOption {
            label: "AAAA (IPv6)".to_string(),
            value: "aaaa".to_string(),
            description: None,
            icon: Some("public".to_string()),
        },
    ];

    rsx! {
        div { class: "space-y-4 animate-fade-in",
            h3 { class: "text-lg font-semibold text-white border-b border-border pb-2 flex items-center gap-2",
                span { class: "material-symbols-outlined text-blue-400", "dns" }
                "Slipstream-Plus"
            }

            div { class: "p-4 bg-bg-tertiary rounded border border-border space-y-4",
                p { class: "text-xs text-gray-400",
                    "Hides high-bandwidth QUIC streams inside standard DNS queries and responses."
                }

                TextInput {
                    label: Some("Root Domain".to_string()),
                    value: root_domain,
                    placeholder: Some("dns.example.com".to_string()),
                }

                div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                    ChoiceBox {
                        label: Some("Record Type".to_string()),
                        value: record_type,
                        options: record_options,
                    }

                    NumberInput {
                        label: Some("UDP Frag Limit (bytes)".to_string()),
                        value: udp_frag_limit,
                        min: Some(512),
                        max: Some(1450),
                        description: Some("1280 safe for IPv6".to_string()),
                    }
                }
            }
        }
    }
}
