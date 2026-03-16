//! Byte Counter Component
//!
//! Animated counter for traffic statistics.

use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ByteCounterProps {
    /// Value in bytes
    pub value: u64,
    /// Color class (e.g. text-green-400)
    #[props(default)]
    pub color_class: String,
}

#[component]
pub fn ByteCounter(props: ByteCounterProps) -> Element {
    // Determine unit
    let (val, unit) = if props.value >= 1024 * 1024 * 1024 {
        (props.value as f64 / 1024.0 / 1024.0 / 1024.0, "GB")
    } else if props.value >= 1024 * 1024 {
        (props.value as f64 / 1024.0 / 1024.0, "MB")
    } else if props.value >= 1024 {
        (props.value as f64 / 1024.0, "KB")
    } else {
        (props.value as f64, "B")
    };

    rsx! {
        div { class: "font-mono font-bold {props.color_class} flex items-baseline gap-1",
            span { class: "text-lg", "{val:.2}" }
            span { class: "text-xs opacity-70", "{unit}" }
        }
    }
}
