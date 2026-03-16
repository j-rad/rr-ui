use crate::ui::components::forms::{ChoiceBox, ChoiceBoxOption, NumberInput, Switch};
use crate::ui::components::secret_generator::{SecretGenerator, SecretType};
use dioxus::prelude::*;

#[component]
pub fn FlowJForm(
    uuid: Signal<String>,
    padding: Signal<i64>,
    jitter: Signal<i64>,
    port_count: Signal<i64>,
    port_type: Signal<Option<String>>,
    congestion: Signal<Option<String>>,
    rotation_frequency: Signal<i64>,
    multiport_enabled: Signal<bool>,
) -> Element {
    let port_type_options = vec![
        ChoiceBoxOption {
            label: "Static Range".to_string(),
            value: "static_range".to_string(),
            description: None,
            icon: Some("linear_scale".to_string()),
        },
        ChoiceBoxOption {
            label: "Random Dynamic".to_string(),
            value: "random_dynamic".to_string(),
            description: None,
            icon: Some("shuffle".to_string()),
        },
    ];

    let congestion_options = vec![
        ChoiceBoxOption {
            label: "Cubic".to_string(),
            value: "cubic".to_string(),
            description: None,
            icon: Some("trending_up".to_string()),
        },
        ChoiceBoxOption {
            label: "BBR".to_string(),
            value: "bbr".to_string(),
            description: None,
            icon: Some("speed".to_string()),
        },
    ];

    rsx! {
        div { class: "space-y-4 animate-fade-in",
            h3 { class: "text-lg font-semibold text-white border-b border-border pb-2 flex items-center gap-2",
                span { class: "material-symbols-outlined text-primary", "water" }
                "Flow-J Settings"
            }

            SecretGenerator {
                label: "Client UUID".to_string(),
                secret_type: SecretType::Uuid,
                value: uuid,
            }

                div { class: "p-4 bg-bg-tertiary rounded border border-border space-y-4",
                    div { class: "flex items-center justify-between",
                        h4 { class: "text-sm font-semibold text-gray-300 flex items-center gap-2",
                            span { class: "material-symbols-outlined text-[16px]", "hub" }
                            "Multiport Configuration"
                        }
                        Switch {
                            label: Some("Enable Multiport Pool".to_string()),
                            value: multiport_enabled,
                        }
                    }

                    if multiport_enabled() {
                        div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                            div {
                                label { class: "block text-sm font-medium text-gray-400 mb-1",
                                    "Socket Pool Size ({port_count})"
                                }
                        input {
                            r#type: "range",
                            min: "1",
                            max: "64",
                            value: "{port_count}",
                            class: "w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer accent-primary",
                            oninput: move |e| {
                                if let Ok(val) = e.value().parse::<i64>() {
                                    port_count.set(val);
                                }
                            }
                        }
                        div { class: "flex justify-between text-xs text-gray-500 mt-1",
                            span { "1" }
                            span { "32" }
                            span { "64" }
                        }
                    }

                    ChoiceBox {
                        label: Some("Port Type".to_string()),
                        value: port_type,
                        options: port_type_options,
                    }
                }

                        div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                            NumberInput {
                                label: Some("Rotation Frequency (min)".to_string()),
                                value: rotation_frequency,
                                min: Some(1),
                                max: Some(1440),
                                description: Some("Dynamic port rotation interval".to_string()),
                            }
                        }
                    }
                }

            div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                NumberInput {
                    label: Some("Padding (0-3)".to_string()),
                    value: padding,
                    min: Some(0),
                    max: Some(3),
                    description: Some("0:None 1:Rand 2:PKCS7 3:Zero".to_string()),
                }

                NumberInput {
                    label: Some("Jitter (ms)".to_string()),
                    value: jitter,
                    min: Some(0),
                    max: Some(1000),
                    description: Some("Random delay injection".to_string()),
                }
            }

            ChoiceBox {
                label: Some("Congestion Control".to_string()),
                value: congestion,
                options: congestion_options,
            }
        }
    }
}
