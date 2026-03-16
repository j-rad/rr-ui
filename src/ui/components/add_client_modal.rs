//! Add Client Modal Component

use crate::ui::components::forms::{DatePicker, NumberInput, TextInput};
use crate::ui::components::modal::Modal;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct AddClientModalProps {
    pub open: Signal<bool>,
    pub inbound_id: i64,
    #[props(default)]
    pub on_close: Option<EventHandler<()>>,
    #[props(default)]
    pub on_save: Option<EventHandler<()>>,
}

#[component]
pub fn AddClientModal(props: AddClientModalProps) -> Element {
    let mut email = use_signal(|| String::new());
    let mut uuid = use_signal(|| String::new());
    let mut total_gb = use_signal(|| 0i64);
    let mut expiry_time = use_signal(|| None::<i64>);
    let mut open = props.open;

    let generate_uuid = move |_| {
        // Generate UUID v4 using random bytes
        let mut bytes = [0u8; 16];
        if let Ok(_) = getrandom::fill(&mut bytes) {
            // Set version to 4 (random)
            bytes[6] = (bytes[6] & 0x0f) | 0x40;
            // Set variant to RFC 4122
            bytes[8] = (bytes[8] & 0x3f) | 0x80;

            let uuid_str = format!(
                "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
                bytes[0],
                bytes[1],
                bytes[2],
                bytes[3],
                bytes[4],
                bytes[5],
                bytes[6],
                bytes[7],
                bytes[8],
                bytes[9],
                bytes[10],
                bytes[11],
                bytes[12],
                bytes[13],
                bytes[14],
                bytes[15]
            );
            uuid.set(uuid_str);
        }
    };

    let handle_save = move |_| {
        if let Some(ref handler) = props.on_save {
            handler.call(());
        }
        open.set(false);
    };

    rsx! {
        Modal {
            open: props.open,
            title: "Add Client".to_string(),
            width: "500px".to_string(),
            on_close: props.on_close.clone(),
            footer: rsx! {
                div { class: "flex items-center justify-end gap-3",
                    button {
                        class: "px-4 py-2 bg-transparent hover:bg-white/5 border border-border text-gray-300 text-sm font-medium rounded transition",
                        onclick: move |_| open.set(false),
                        "Cancel"
                    }
                    button {
                        class: "px-4 py-2 bg-primary hover:bg-primary-hover text-white text-sm font-medium rounded transition shadow-sm",
                        onclick: handle_save,
                        "Add Client"
                    }
                }
            },

            div { class: "space-y-4",
                TextInput {
                    label: Some("Email".to_string()),
                    value: email,
                    placeholder: Some("user@example.com".to_string()),
                    required: true,
                }

                div { class: "flex items-end gap-2",
                    div { class: "flex-1",
                        TextInput {
                            label: Some("UUID".to_string()),
                            value: uuid,
                            placeholder: Some("Auto-generated...".to_string()),
                        }
                    }
                    button {
                        class: "px-4 py-2 bg-primary/10 hover:bg-primary/20 border border-primary text-primary text-sm font-medium rounded transition mb-1",
                        onclick: generate_uuid,
                        "Generate"
                    }
                }

                NumberInput {
                    label: Some("Total GB".to_string()),
                    value: total_gb,
                    min: Some(0),
                    unit: Some("GB".to_string()),
                    description: Some("0 for unlimited".to_string()),
                }

                DatePicker {
                    label: Some("Expiry Date".to_string()),
                    value: expiry_time,
                    description: Some("Leave empty for no expiration".to_string()),
                }
            }
        }
    }
}
