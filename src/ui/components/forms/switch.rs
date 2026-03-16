//! Switch Component
//!
//! Toggle switch with animated transition.

use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct SwitchProps {
    /// Current value
    pub value: Signal<bool>,
    /// Label text
    #[props(default)]
    pub label: Option<String>,
    /// Label position (left or right)
    #[props(default = "right".to_string())]
    pub label_position: String,
    /// Whether the switch is disabled
    #[props(default = false)]
    pub disabled: bool,
    /// Custom change handler
    #[props(default)]
    pub on_change: Option<EventHandler<bool>>,
}

#[component]
pub fn Switch(props: SwitchProps) -> Element {
    let mut value = props.value;
    let on_change = props.on_change.clone();

    let handle_click = move |_| {
        if !props.disabled {
            let new_value = !value();
            value.set(new_value);
            if let Some(ref handler) = on_change {
                handler.call(new_value);
            }
        }
    };

    let switch_bg = if value() { "bg-primary" } else { "bg-gray-600" };

    let switch_translate = if value() {
        "translate-x-5"
    } else {
        "translate-x-0"
    };

    let wrapper_classes = if props.label_position == "left" {
        "flex flex-row-reverse items-center gap-3"
    } else {
        "flex items-center gap-3"
    };

    rsx! {
        div {
            class: if props.disabled { format!("{} opacity-50 cursor-not-allowed", wrapper_classes) } else { format!("{} cursor-pointer", wrapper_classes) },
            onclick: handle_click,

            // Switch
            div {
                class: format!("relative inline-block w-11 h-6 {} rounded-full transition-colors duration-200 ease-in-out", switch_bg),
                div {
                    class: "absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full shadow transition-transform duration-200 ease-in-out {switch_translate}",
                }
            }

            // Label
            if let Some(label) = props.label {
                span { class: "text-sm font-medium text-gray-300 select-none", "{label}" }
            }
        }
    }
}
