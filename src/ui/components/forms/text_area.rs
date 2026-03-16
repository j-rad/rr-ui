//! TextArea Component
//!
//! Multi-line text area with optional copy button and character counter.

use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct TextAreaProps {
    /// Input label
    #[props(default)]
    pub label: Option<String>,
    /// Current value
    pub value: Signal<String>,
    /// Placeholder text
    #[props(default)]
    pub placeholder: Option<String>,
    /// Number of rows
    #[props(default = 4)]
    pub rows: u32,
    /// Whether the input is disabled
    #[props(default = false)]
    pub disabled: bool,
    /// Error message to display
    #[props(default)]
    pub error: Option<String>,
    /// Optional description/hint text
    #[props(default)]
    pub description: Option<String>,
    /// Whether to use monospace font
    #[props(default = false)]
    pub monospace: bool,
    /// Whether to show character count
    #[props(default = false)]
    pub show_count: bool,
    /// Maximum character length
    #[props(default)]
    pub max_length: Option<usize>,
    /// Whether the input is required
    #[props(default = false)]
    pub required: bool,
    /// Whether to show copy button
    #[props(default = false)]
    pub show_copy: bool,
    /// Custom change handler
    #[props(default)]
    pub on_change: Option<EventHandler<String>>,
}

#[component]
pub fn TextArea(props: TextAreaProps) -> Element {
    let mut value = props.value;
    let mut copied = use_signal(|| false);
    let on_change = props.on_change.clone();

    let handle_input = move |evt: Event<FormData>| {
        let new_value = evt.value();

        // Apply max length if specified
        let final_value = if let Some(max) = props.max_length {
            if new_value.len() > max {
                new_value[..max].to_string()
            } else {
                new_value
            }
        } else {
            new_value
        };

        value.set(final_value.clone());
        if let Some(ref handler) = on_change {
            handler.call(final_value);
        }
    };

    let handle_copy = move |_| {
        // In a real implementation, use web_sys to copy to clipboard
        // For now, just show visual feedback
        copied.set(true);

        // Reset after 2 seconds (would need spawn_local in real impl)
    };

    let textarea_classes = if props.error.is_some() {
        format!(
            "w-full bg-[#0a0a0a] border border-red-500 text-gray-300 text-sm rounded px-3 py-2 focus:outline-none focus:border-red-500 focus:ring-1 focus:ring-red-500 transition-all resize-vertical {}",
            if props.monospace { "font-mono" } else { "" }
        )
    } else {
        format!(
            "w-full bg-[#0a0a0a] border border-border text-gray-300 text-sm rounded px-3 py-2 focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary transition-all placeholder-gray-600 resize-vertical {}",
            if props.monospace { "font-mono" } else { "" }
        )
    };

    let char_count = value().len();
    let count_color = if let Some(max) = props.max_length {
        if char_count > max {
            "text-red-500"
        } else if char_count > (max * 9 / 10) {
            "text-yellow-500"
        } else {
            "text-gray-500"
        }
    } else {
        "text-gray-500"
    };

    rsx! {
        div { class: "flex flex-col gap-1.5",
            // Label row with copy button
            div { class: "flex items-center justify-between",
                if let Some(label) = props.label {
                    label {
                        class: "block text-sm font-medium text-gray-400",
                        "{label}"
                        if props.required {
                            span { class: "text-red-500 ml-1", "*" }
                        }
                    }
                }

                if props.show_copy {
                    button {
                        class: "text-xs text-gray-500 hover:text-primary transition-colors flex items-center gap-1",
                        r#type: "button",
                        onclick: handle_copy,
                        span { class: "material-symbols-outlined text-[14px]",
                            if copied() { "check" } else { "content_copy" }
                        }
                        if copied() { "Copied!" } else { "Copy" }
                    }
                }
            }

            // TextArea
            textarea {
                class: "{textarea_classes}",
                value: "{value()}",
                placeholder: props.placeholder.as_deref().unwrap_or(""),
                disabled: props.disabled,
                rows: "{props.rows}",
                maxlength: "{props.max_length.map(|v| v.to_string()).unwrap_or_default()}",
                oninput: handle_input,
            }

            // Footer with description/error and character count
            div { class: "flex items-center justify-between",
                if let Some(error) = props.error {
                    p { class: "text-xs text-red-500", "{error}" }
                } else if let Some(desc) = props.description {
                    p { class: "text-xs text-gray-500", "{desc}" }
                } else {
                    span {}
                }

                if props.show_count {
                    span { class: "text-xs {count_color}",
                        "{char_count}"
                        if let Some(max) = props.max_length {
                            " / {max}"
                        }
                    }
                }
            }
        }
    }
}
