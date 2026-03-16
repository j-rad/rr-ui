use crate::ui::components::forms::{ChoiceBox, ChoiceBoxOption};
use crate::ui::components::secret_generator::{SecretGenerator, SecretType};
use dioxus::prelude::*;

#[component]
pub fn VlessForm(uuid: Signal<String>, flow: Signal<Option<String>>) -> Element {
    let flow_options = vec![
        ChoiceBoxOption {
            value: "".to_string(),
            label: "None".to_string(),
            description: None,
            icon: None,
        },
        ChoiceBoxOption {
            value: "xtls-rprx-vision".to_string(),
            label: "xtls-rprx-vision".to_string(),
            description: Some("Recommended for VLESS".to_string()),
            icon: None,
        },
        ChoiceBoxOption {
            value: "xtls-rprx-vision-udp443".to_string(),
            label: "xtls-rprx-vision-udp443".to_string(),
            description: None,
            icon: None,
        },
    ];

    rsx! {
        div { class: "space-y-4",
            h3 { class: "text-lg font-semibold text-white border-b border-border pb-2 flex items-center gap-2",
                span { class: "material-symbols-outlined text-primary", "key" }
                "VLESS Credentials"
            }

            SecretGenerator {
                label: "Client UUID".to_string(),
                secret_type: SecretType::Uuid,
                value: uuid,
            }

            ChoiceBox {
                label: Some("Flow".to_string()),
                value: flow,
                options: flow_options,
                description: Some("XTLS flow control for VLESS".to_string()),
            }
        }
    }
}
