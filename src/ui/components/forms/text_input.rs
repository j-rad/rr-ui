//! TextInput Component
//!
//! Reusable text input with label, validation, and styling.

use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct TextInputProps {
    /// Input label
    #[props(default)]
    pub label: Option<String>,
    /// Current value
    pub value: Signal<String>,
    /// Placeholder text
    #[props(default)]
    pub placeholder: Option<String>,
    /// Whether the input is disabled
    #[props(default = false)]
    pub disabled: bool,
    /// Error message to display
    #[props(default)]
    pub error: Option<String>,
    /// Optional description/hint text
    #[props(default)]
    pub description: Option<String>,
    /// Input type (text, email, password, etc.)
    #[props(default = "text".to_string())]
    pub input_type: String,
    /// Optional prefix icon
    #[props(default)]
    pub prefix_icon: Option<String>,
    /// Optional suffix icon
    #[props(default)]
    pub suffix_icon: Option<String>,
    /// Whether the input is required
    #[props(default = false)]
    pub required: bool,
    /// Custom change handler
    #[props(default)]
    pub on_change: Option<EventHandler<String>>,
}

#[component]
pub fn TextInput(props: TextInputProps) -> Element {
    let mut value = props.value;
    let on_change = props.on_change.clone();

    let handle_input = move |evt: Event<FormData>| {
        let new_value = evt.value();
        value.set(new_value.clone());
        if let Some(ref handler) = on_change {
            handler.call(new_value);
        }
    };

    let input_classes = if props.error.is_some() {
        "w-full bg-[#0a0a0a] border border-red-500 text-gray-300 text-sm rounded px-3 py-2 focus:outline-none focus:border-red-500 focus:ring-1 focus:ring-red-500 transition-all"
    } else {
        "w-full bg-[#0a0a0a] border border-border text-gray-300 text-sm rounded px-3 py-2 focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary transition-all placeholder-gray-600"
    };

    rsx! {
        div { class: "flex flex-col gap-1.5",
            // Label
            if let Some(label) = props.label {
                label {
                    class: "block text-sm font-medium text-gray-400",
                    "{label}"
                    if props.required {
                        span { class: "text-red-500 ml-1", "*" }
                    }
                }
            }

            // Input wrapper with icons
            div { class: "relative",
                // Prefix icon
                if let Some(ref prefix) = props.prefix_icon {
                    span {
                        class: "absolute left-3 top-1/2 -translate-y-1/2 material-symbols-outlined text-gray-500 text-[18px]",
                        "{prefix}"
                    }
                }

                // Input field
                {
                    let mut class_parts = vec![input_classes.to_string()];
                    if props.prefix_icon.is_some() {
                        class_parts.push("pl-10".to_string());
                    }
                    if props.suffix_icon.is_some() {
                        class_parts.push("pr-10".to_string());
                    }
                    let input_class = class_parts.join(" ");
                    rsx! {
                        input {
                            class: input_class,
                            r#type: "{props.input_type}",
                            value: "{value()}",
                            placeholder: props.placeholder.as_deref().unwrap_or(""),
                            disabled: props.disabled,
                            oninput: handle_input,
                        }
                    }
                }

                // Suffix icon
                if let Some(suffix) = props.suffix_icon {
                    span {
                        class: "absolute right-3 top-1/2 -translate-y-1/2 material-symbols-outlined text-gray-500 text-[18px]",
                        "{suffix}"
                    }
                }
            }

            // Description or Error
            if let Some(error) = props.error {
                p { class: "text-xs text-red-500 mt-0.5", "{error}" }
            } else if let Some(desc) = props.description {
                p { class: "text-xs text-gray-500 mt-0.5", "{desc}" }
            }
        }
    }
}
