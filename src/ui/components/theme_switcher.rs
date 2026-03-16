//! Theme Switcher Component

use crate::ui::state::GlobalState;
use dioxus::prelude::*;

#[component]
pub fn ThemeSwitcher() -> Element {
    let state = use_context::<GlobalState>();
    let mut theme = state.theme;

    let toggle_theme = move |_| {
        let new_theme = if theme() == "dark" {
            "light".to_string()
        } else {
            "dark".to_string()
        };
        theme.set(new_theme);
    };

    let is_dark = theme() == "dark";

    rsx! {
        button {
            class: "p-2 rounded-lg hover:bg-white/5 transition-colors",
            onclick: toggle_theme,
            title: if is_dark { "Switch to Light Mode" } else { "Switch to Dark Mode" },

            if is_dark {
                span { class: "material-symbols-outlined text-yellow-400", "light_mode" }
            } else {
                span { class: "material-symbols-outlined text-gray-600", "dark_mode" }
            }
        }
    }
}
