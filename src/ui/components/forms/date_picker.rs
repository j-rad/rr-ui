//! DatePicker Component
//!
//! Date/time picker with Persian (Jalaali) calendar support.

use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct DatePickerProps {
    /// Input label
    #[props(default)]
    pub label: Option<String>,
    /// Current value (Unix timestamp in milliseconds)
    pub value: Signal<Option<i64>>,
    /// Whether to use Persian calendar
    #[props(default = false)]
    pub persian: bool,
    /// Whether to include time picker
    #[props(default = false)]
    pub include_time: bool,
    /// Minimum date (Unix timestamp)
    #[props(default)]
    pub min_date: Option<i64>,
    /// Maximum date (Unix timestamp)
    #[props(default)]
    pub max_date: Option<i64>,
    /// Placeholder text
    #[props(default = "Select date...".to_string())]
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
    /// Custom change handler
    #[props(default)]
    pub on_change: Option<EventHandler<i64>>,
}

#[component]
pub fn DatePicker(props: DatePickerProps) -> Element {
    let mut value = props.value;
    let mut is_open = use_signal(|| false);
    let on_change = props.on_change.clone();

    let toggle_picker = move |_| {
        if !props.disabled {
            is_open.set(!is_open());
        }
    };

    let clear_date = move |_| {
        value.set(None);
        if let Some(ref handler) = on_change {
            handler.call(0);
        }
    };

    // Format timestamp for display
    let formatted_date = value().map(|ts| {
        // Simple formatting - in production, use chrono or jalaali crate
        if props.persian {
            format!("Persian: {}", ts) // Placeholder
        } else {
            format!("Date: {}", ts) // Placeholder
        }
    });

    let display_date = formatted_date
        .clone()
        .unwrap_or_else(|| props.placeholder.clone());
    let has_date = formatted_date.is_some();

    let input_classes = if props.error.is_some() {
        "w-full bg-[#0a0a0a] border border-red-500 text-gray-300 text-sm rounded px-3 py-2 cursor-pointer flex items-center justify-between hover:bg-[#0f0f0f] transition-colors"
    } else {
        "w-full bg-[#0a0a0a] border border-border text-gray-300 text-sm rounded px-3 py-2 cursor-pointer flex items-center justify-between hover:bg-[#0f0f0f] transition-colors"
    };

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

            // Date input trigger
            div { class: "flex items-center gap-2",
                div {
                    class: if props.disabled { format!("{} opacity-50 cursor-not-allowed", input_classes) } else { input_classes.to_string() },
                    onclick: toggle_picker,

                    div { class: "flex items-center gap-2",
                        span { class: "material-symbols-outlined text-gray-500 text-[20px]", "calendar_today" }
                        span {
                            class: if has_date { "text-gray-300" } else { "text-gray-600" },
                            "{display_date}"
                        }
                    }

                    if value().is_some() && !props.disabled {
                        button {
                            class: "text-gray-500 hover:text-red-500 transition-colors",
                            r#type: "button",
                            onclick: clear_date,
                            span { class: "material-symbols-outlined text-[18px]", "close" }
                        }
                    }
                }
            }

            // Calendar popup (simplified - full implementation would use a calendar library)
            if is_open() && !props.disabled {
                div {
                    class: "absolute top-full left-0 mt-1 bg-[#1a1a1a] border border-border rounded shadow-lg p-4 z-50 min-w-[280px]",

                    div { class: "text-center text-gray-400 text-sm mb-4",
                        if props.persian {
                            "Persian Calendar (Jalaali)"
                        } else {
                            "Gregorian Calendar"
                        }
                    }

                    // Placeholder for actual calendar grid
                    div { class: "text-center text-gray-500 text-xs py-8",
                        "Calendar implementation pending"
                        br {}
                        "Use native date input for now"
                    }

                    // Native date input fallback
                    input {
                        class: "w-full bg-[#0a0a0a] border border-border text-gray-300 text-sm rounded px-3 py-2 focus:outline-none focus:border-primary",
                        r#type: if props.include_time { "datetime-local" } else { "date" },
                        onchange: move |evt: Event<FormData>| {
                            // Parse date and convert to timestamp
                            // This is a simplified implementation
                            let timestamp = 0i64; // Placeholder
                            value.set(Some(timestamp));
                            is_open.set(false);
                            if let Some(ref handler) = on_change {
                                handler.call(timestamp);
                            }
                        },
                    }

                    if props.persian {
                        div { class: "mt-2 text-xs text-gray-500 text-center",
                            "Note: Full Persian calendar requires jalaali crate integration"
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
