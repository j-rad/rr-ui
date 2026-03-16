// src/ui/components/theme_toggle.rs
//! Theme Toggle Button Component
//!
//! Provides a button to toggle between light and dark themes

use super::theme_provider::{Theme, use_theme, use_toggle_theme};
use dioxus::prelude::*;

#[component]
pub fn ThemeToggle() -> Element {
    let theme = use_theme();
    let toggle = use_toggle_theme();

    rsx! {
        button {
            class: "theme-toggle transition-smooth",
            onclick: move |_| toggle(),
            title: "Toggle theme ({theme().as_str()})",

            if theme() == Theme::Dark {
                i { class: "fas fa-sun" }
                span { class: "sr-only", "Switch to light mode" }
            } else {
                i { class: "fas fa-moon" }
                span { class: "sr-only", "Switch to dark mode" }
            }
        }
    }
}

/// Theme selector dropdown (for settings page)
#[component]
pub fn ThemeSelector() -> Element {
    let theme = use_theme();
    let set_theme = super::theme_provider::use_set_theme();

    rsx! {
        div { class: "theme-selector",
            label {
                r#for: "theme-select",
                "Theme"
            }
            select {
                id: "theme-select",
                class: "form-select",
                value: "{theme().as_str()}",
                onchange: move |e| {
                    let value = e.value();
                    set_theme(Theme::from_str(&value));
                },

                option { value: "light", "Light" }
                option { value: "dark", "Dark" }
                option { value: "system", "System" }
            }
        }
    }
}
