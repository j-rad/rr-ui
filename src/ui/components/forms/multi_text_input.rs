//! MultiTextInput Component
//!
//! Dynamic list of text inputs with add/remove functionality.

use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct MultiTextInputProps {
    /// Input label
    #[props(default)]
    pub label: Option<String>,
    /// Current values
    pub value: Signal<Vec<String>>,
    /// Placeholder text for new entries
    #[props(default = "Enter value...".to_string())]
    pub placeholder: String,
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
    /// Minimum number of entries
    #[props(default = 0)]
    pub min_entries: usize,
    /// Maximum number of entries
    #[props(default)]
    pub max_entries: Option<usize>,
    /// Custom change handler
    #[props(default)]
    pub on_change: Option<EventHandler<Vec<String>>>,
}

#[component]
pub fn MultiTextInput(props: MultiTextInputProps) -> Element {
    let mut value = props.value;
    let on_change = props.on_change.clone();

    let notify_change = move || {
        if let Some(ref handler) = on_change {
            handler.call(value());
        }
    };

    let add_entry = move |_| {
        if props
            .max_entries
            .map(|max| value().len() < max)
            .unwrap_or(true)
        {
            let mut current = value();
            current.push(String::new());
            value.set(current);
            notify_change();
        }
    };

    let mut remove_entry = move |index: usize| {
        if value().len() > props.min_entries {
            let mut current = value();
            current.remove(index);
            value.set(current);
            notify_change();
        }
    };

    let mut update_entry = move |index: usize, new_value: String| {
        let mut current = value();
        if index < current.len() {
            current[index] = new_value;
            value.set(current);
            notify_change();
        }
    };

    let can_add = props
        .max_entries
        .map(|max| value().len() < max)
        .unwrap_or(true);
    let can_remove = value().len() > props.min_entries;

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

            // Entries
            div { class: "space-y-2",
                for (index , entry) in value().iter().enumerate() {
                    {
                        let idx = index;
                        let entry_value = entry.clone();
                        rsx! {
                            div {
                                key: "{idx}",
                                class: "flex items-center gap-2",

                                input {
                                    class: "flex-1 bg-[#0a0a0a] border border-border text-gray-300 text-sm rounded px-3 py-2 focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary transition-all placeholder-gray-600",
                                    r#type: "text",
                                    value: "{entry_value}",
                                    placeholder: "{props.placeholder}",
                                    disabled: props.disabled,
                                    oninput: move |evt: Event<FormData>| {
                                        update_entry(idx, evt.value());
                                    },
                                }

                                if can_remove && !props.disabled {
                                    button {
                                        class: "p-2 text-gray-500 hover:text-red-500 hover:bg-red-500/10 rounded transition-colors",
                                        r#type: "button",
                                        onclick: move |_| remove_entry(idx),
                                        span { class: "material-symbols-outlined text-[20px]", "close" }
                                    }
                                }
                            }
                        }
                    }
                }

                // Add button
                if can_add && !props.disabled {
                    button {
                        class: "flex items-center gap-2 px-3 py-2 text-sm text-primary hover:bg-primary/10 border border-dashed border-border hover:border-primary rounded transition-colors",
                        r#type: "button",
                        onclick: add_entry,
                        span { class: "material-symbols-outlined text-[18px]", "add" }
                        "Add Entry"
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
