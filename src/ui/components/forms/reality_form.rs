use crate::ui::components::forms::{
    ChoiceBox, ChoiceBoxOption, MultiTextInput, Switch, TextArea, TextInput,
};
use dioxus::prelude::*;

#[component]
pub fn RealityForm(
    dest: Signal<String>,
    sni: Signal<Vec<String>>,
    short_ids: Signal<Vec<String>>,
    private_key: Signal<String>,
    public_key: Signal<String>,
    pqc: Signal<Option<String>>,
    fingerprint: Signal<Option<String>>,
    stealth: Signal<bool>,
    generate_keys: EventHandler<()>,
) -> Element {
    let pqc_options = vec![
        ChoiceBoxOption {
            value: "".to_string(),
            label: "None".to_string(),
            description: None,
            icon: None,
        },
        ChoiceBoxOption {
            value: "Kyber768".to_string(),
            label: "Kyber-768".to_string(),
            description: Some("NIST Level 3 KEM".to_string()),
            icon: None,
        },
        ChoiceBoxOption {
            value: "Kyber1024".to_string(),
            label: "Kyber-1024".to_string(),
            description: Some("NIST Level 5 KEM".to_string()),
            icon: None,
        },
        ChoiceBoxOption {
            value: "Dilithium2".to_string(),
            label: "Dilithium-2".to_string(),
            description: Some("NIST Level 2 Sig".to_string()),
            icon: None,
        },
        ChoiceBoxOption {
            value: "Dilithium3".to_string(),
            label: "Dilithium-3".to_string(),
            description: Some("NIST Level 3 Sig".to_string()),
            icon: None,
        },
        ChoiceBoxOption {
            value: "Dilithium5".to_string(),
            label: "Dilithium-5".to_string(),
            description: Some("NIST Level 5 Sig".to_string()),
            icon: None,
        },
    ];

    rsx! {
        div { class: "space-y-4",
            h3 { class: "text-lg font-semibold text-white border-b border-border pb-2 flex items-center gap-2",
                span { class: "material-symbols-outlined text-green-400", "verified_user" }
                "REALITY Settings"
                span { class: "px-2 py-0.5 bg-green-500/20 text-green-400 text-xs rounded font-bold", "ADVANCED" }
            }

            TextInput {
                label: Some("Dest".to_string()),
                value: dest,
                placeholder: Some("www.google.com:443".to_string()),
                description: Some("Camouflage destination server".to_string()),
            }

            MultiTextInput {
                label: Some("SNI List".to_string()),
                value: sni,
                placeholder: "example.com".to_string(),
                description: Some("Server Name Indication domains".to_string()),
            }

            MultiTextInput {
                label: Some("Short IDs".to_string()),
                value: short_ids,
                placeholder: "Hex string...".to_string(),
                description: Some("Short ID list (1-8 bytes hex)".to_string()),
            }

            div { class: "p-4 bg-[#1a1a1a] rounded border border-border space-y-4",
                div { class: "flex items-center justify-between",
                    span { class: "text-sm font-medium text-gray-300", "Reality Keys" }
                    button {
                        class: "flex items-center gap-2 px-3 py-1.5 text-sm bg-primary/10 text-primary rounded-lg hover:bg-primary/20 transition-colors",
                        onclick: move |_| generate_keys.call(()),
                        span { class: "material-symbols-outlined text-[16px]", "autorenew" }
                        "Generate Pair"
                    }
                }

                TextArea {
                    label: Some("Private Key".to_string()),
                    value: private_key,
                    rows: 2,
                    monospace: true,
                    show_copy: true,
                }

                TextArea {
                    label: Some("Public Key".to_string()),
                    value: public_key,
                    rows: 2,
                    monospace: true,
                    show_copy: true,
                }
            }

            div { class: "grid grid-cols-2 gap-4",
                ChoiceBox {
                    label: Some("PQC Cipher".to_string()),
                    value: pqc,
                    options: pqc_options,
                    description: Some("Post-Quantum Cryptography Matrix".to_string()),
                }

                ChoiceBox {
                    label: Some("uTLS Fingerprint".to_string()),
                    value: fingerprint,
                    options: vec![
                        ChoiceBoxOption { value: "chrome".to_string(), label: "Chrome".to_string(), description: None, icon: None },
                        ChoiceBoxOption { value: "firefox".to_string(), label: "Firefox".to_string(), description: None, icon: None },
                        ChoiceBoxOption { value: "safari".to_string(), label: "Safari".to_string(), description: None, icon: None },
                        ChoiceBoxOption { value: "randomized".to_string(), label: "Randomized".to_string(), description: None, icon: None },
                    ],
                    description: Some("Client Hello camouflage".to_string()),
                }


                Switch {
                    label: Some("Stealth Handshake".to_string()),
                    value: stealth,
                }
            }
        }
    }
}
