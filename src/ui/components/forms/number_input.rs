//! NumberInput Component
//!
//! Number input with increment/decrement controls.

use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct NumberInputProps {
    /// Input label
    #[props(default)]
    pub label: Option<String>,
    /// Current value
    pub value: Signal<i64>,
    /// Minimum value
    #[props(default)]
    pub min: Option<i64>,
    /// Maximum value
    #[props(default)]
    pub max: Option<i64>,
    /// Step increment
    #[props(default = 1)]
    pub step: i64,
    /// Optional unit suffix (e.g., "GB", "ms")
    #[props(default)]
    pub unit: Option<String>,
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
    /// Whether the input is required
    #[props(default = false)]
    pub required: bool,
    /// Custom change handler
    #[props(default)]
    pub on_change: Option<EventHandler<i64>>,
}

#[component]
pub fn NumberInput(props: NumberInputProps) -> Element {
    let mut value = props.value;
    let on_change = props.on_change.clone();

    let mut validate_and_set = move |new_value: i64| {
        let mut validated = new_value;

        // Apply min constraint
        if let Some(min) = props.min {
            validated = validated.max(min);
        }

        // Apply max constraint
        if let Some(max) = props.max {
            validated = validated.min(max);
        }

        value.set(validated);
        if let Some(ref handler) = on_change {
            handler.call(validated);
        }
    };

    let handle_input = move |evt: Event<FormData>| {
        if let Ok(parsed) = evt.value().parse::<i64>() {
            validate_and_set(parsed);
        }
    };

    let increment = move |_| {
        let new_value = value() + props.step;
        validate_and_set(new_value);
    };

    let decrement = move |_| {
        let new_value = value() - props.step;
        validate_and_set(new_value);
    };

    let input_classes = if props.error.is_some() {
        "w-full bg-[#0a0a0a] border border-red-500 text-gray-300 text-sm rounded-l px-3 py-2 focus:outline-none focus:border-red-500 focus:ring-1 focus:ring-red-500 transition-all"
    } else {
        "w-full bg-[#0a0a0a] border border-border text-gray-300 text-sm rounded-l px-3 py-2 focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary transition-all placeholder-gray-600"
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

            // Input wrapper with controls
            div { class: "flex items-center",
                // Input field
                input {
                    class: "{input_classes}",
                    r#type: "number",
                    value: "{value()}",
                    placeholder: props.placeholder.as_deref().unwrap_or(""),
                    disabled: props.disabled,
                    min: "{props.min.map(|v| v.to_string()).unwrap_or_default()}",
                    max: "{props.max.map(|v| v.to_string()).unwrap_or_default()}",
                    step: "{props.step}",
                    oninput: handle_input,
                }

                // Controls
                div { class: "flex flex-col border border-l-0 border-border rounded-r overflow-hidden",
                    button {
                        class: "px-2 py-0.5 bg-[#1a1a1a] hover:bg-[#2a2a2a] border-b border-border text-gray-400 hover:text-white transition-colors disabled:opacity-50 disabled:cursor-not-allowed",
                        r#type: "button",
                        disabled: props.disabled || props.max.map(|max| value() >= max).unwrap_or(false),
                        onclick: increment,
                        span { class: "material-symbols-outlined text-[14px]", "expand_less" }
                    }
                    button {
                        class: "px-2 py-0.5 bg-[#1a1a1a] hover:bg-[#2a2a2a] text-gray-400 hover:text-white transition-colors disabled:opacity-50 disabled:cursor-not-allowed",
                        r#type: "button",
                        disabled: props.disabled || props.min.map(|min| value() <= min).unwrap_or(false),
                        onclick: decrement,
                        span { class: "material-symbols-outlined text-[14px]", "expand_more" }
                    }
                }

                // Unit suffix
                if let Some(unit) = props.unit {
                    span {
                        class: "ml-2 text-sm text-gray-500 font-medium",
                        "{unit}"
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
