use crate::ui::components::forms::{ChoiceBox, ChoiceBoxOption, TextInput, TextArea};
use dioxus::prelude::*;

#[component]
pub fn DbMimicForm(
    target: Signal<Option<String>>,
    fake_db_name: Signal<String>,
    fake_user: Signal<String>,
    startup_payload_hex: Signal<String>,
) -> Element {
    let target_options = vec![
        ChoiceBoxOption {
            label: "PostgreSQL".to_string(),
            value: "postgresql".to_string(),
            description: None,
            icon: Some("database".to_string()),
        },
        ChoiceBoxOption {
            label: "Redis".to_string(),
            value: "redis".to_string(),
            description: None,
            icon: Some("storage".to_string()),
        },
        ChoiceBoxOption {
            label: "MySQL".to_string(),
            value: "mysql".to_string(),
            description: None,
            icon: Some("table_view".to_string()),
        },
    ];

    rsx! {
        div { class: "space-y-4 animate-fade-in",
            h3 { class: "text-lg font-semibold text-white border-b border-border pb-2 flex items-center gap-2",
                span { class: "material-symbols-outlined text-green-400", "database" }
                "Database Mimicry"
            }

            div { class: "p-4 bg-bg-tertiary rounded border border-border space-y-4",
                p { class: "text-xs text-gray-400",
                    "Wraps traffic in valid database wire protocol frames to bypass Deep Packet Inspection (DPI)."
                }

                ChoiceBox {
                    label: Some("Mimic Target".to_string()),
                    value: target,
                    options: target_options,
                }

                div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                    TextInput {
                        label: Some("Fake DB Name".to_string()),
                        value: fake_db_name,
                        placeholder: Some("postgres".to_string()),
                    }

                    TextInput {
                        label: Some("Fake User".to_string()),
                        value: fake_user,
                        placeholder: Some("admin".to_string()),
                    }
                }

                TextArea {
                    label: Some("Startup Payload (Hex)".to_string()),
                    value: startup_payload_hex,
                    rows: 3,
                    placeholder: Some("00 03 00 00 ...".to_string()),
                }
            }
        }
    }
}
