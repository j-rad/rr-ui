//! ChoiceBox Component
//!
//! Dropdown select component with search capability.

use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ChoiceBoxProps<T: Clone + PartialEq + 'static> {
    /// Input label
    #[props(default)]
    pub label: Option<String>,
    /// Current selected value
    pub value: Signal<Option<T>>,
    /// List of options
    pub options: Vec<ChoiceBoxOption<T>>,
    /// Placeholder text
    #[props(default = "Select an option...".to_string())]
    pub placeholder: String,
    /// Whether the select is disabled
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
    pub on_change: Option<EventHandler<T>>,
}

#[derive(Clone, PartialEq)]
pub struct ChoiceBoxOption<T: Clone + PartialEq> {
    pub value: T,
    pub label: String,
    pub description: Option<String>,
    pub icon: Option<String>,
}

#[component]
pub fn ChoiceBox<T: Clone + PartialEq + 'static>(props: ChoiceBoxProps<T>) -> Element {
    let mut value = props.value;
    let mut is_open = use_signal(|| false);
    let on_change = props.on_change.clone();

    let toggle_dropdown = move |_| {
        if !props.disabled {
            is_open.set(!is_open());
        }
    };

    let mut select_option = move |option: ChoiceBoxOption<T>| {
        value.set(Some(option.value.clone()));
        is_open.set(false);
        if let Some(ref handler) = on_change {
            handler.call(option.value);
        }
    };

    let selected_label = value().and_then(|val| {
        props
            .options
            .iter()
            .find(|opt| opt.value == val)
            .map(|opt| opt.label.clone())
    });

    let input_classes = if props.error.is_some() {
        "w-full bg-[#0a0a0a] border border-red-500 text-gray-300 text-sm rounded px-3 py-2 cursor-pointer flex items-center justify-between hover:bg-[#0f0f0f] transition-colors"
    } else {
        "w-full bg-[#0a0a0a] border border-border text-gray-300 text-sm rounded px-3 py-2 cursor-pointer flex items-center justify-between hover:bg-[#0f0f0f] transition-colors"
    };

    let display_label = selected_label
        .clone()
        .unwrap_or_else(|| props.placeholder.clone());
    let has_selection = selected_label.is_some();

    rsx! {
        div { class: "flex flex-col gap-1.5 relative",
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

            // Select trigger
            div {
                class: if props.disabled { format!("{} opacity-50 cursor-not-allowed", input_classes) } else { input_classes.to_string() },
                onclick: toggle_dropdown,

                span {
                    class: if has_selection { "text-gray-300" } else { "text-gray-600" },
                    "{display_label}"
                }

                span {
                    class: if is_open() { "material-symbols-outlined text-gray-500 text-[20px] transition-transform rotate-180" } else { "material-symbols-outlined text-gray-500 text-[20px] transition-transform" },
                    "expand_more"
                }
            }

            // Dropdown menu
            if is_open() && !props.disabled {
                div {
                    class: "absolute top-full left-0 right-0 mt-1 bg-[#1a1a1a] border border-border rounded shadow-lg max-h-60 overflow-y-auto z-50",
                    for option in &props.options {
                        {
                            let opt = option.clone();
                            let is_selected = value().as_ref().map(|v| v == &opt.value).unwrap_or(false);
                            let item_class = if is_selected {
                                "px-3 py-2 hover:bg-[#2a2a2a] cursor-pointer transition-colors bg-primary/10 text-primary"
                            } else {
                                "px-3 py-2 hover:bg-[#2a2a2a] cursor-pointer transition-colors text-gray-300"
                            };
                            rsx! {
                                div {
                                    key: "{opt.label}",
                                    class: item_class,
                                    onclick: move |_| select_option(opt.clone()),

                                    div { class: "font-medium", "{opt.label}" }
                                    if let Some(desc) = &opt.description {
                                        div { class: "text-xs text-gray-500 mt-0.5", "{desc}" }
                                    }
                                }
                            }
                        }
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
