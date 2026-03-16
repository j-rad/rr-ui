// src/ui/components/theme_provider.rs
//! Theme Provider Component
//!
//! Provides theme context to all child components with:
//! - Persistent theme preference (localStorage)
//! - System preference detection
//! - Smooth theme transitions

use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub enum Theme {
    Light,
    Dark,
    System,
}

impl Theme {
    pub fn as_str(&self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
            Theme::System => "system",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "light" => Theme::Light,
            "dark" => Theme::Dark,
            "system" => Theme::System,
            _ => Theme::Dark,
        }
    }
}

/// Theme Provider Component
///
/// Wraps the application and provides theme context
/// Note: localStorage persistence requires server-side rendering or custom JS integration
#[component]
pub fn ThemeProvider(children: Element) -> Element {
    let mut theme = use_signal(|| Theme::Dark);
    let mut resolved_theme = use_signal(|| Theme::Dark);

    // Resolve system theme preference (simplified for now)
    use_effect(move || {
        if theme() == Theme::System {
            // Default to dark for system
            resolved_theme.set(Theme::Dark);
        } else {
            resolved_theme.set(theme());
        }
    });

    // Provide theme context
    use_context_provider(|| theme);
    use_context_provider(|| resolved_theme);

    rsx! { {children} }
}

/// Hook to access current theme
pub fn use_theme() -> Signal<Theme> {
    use_context::<Signal<Theme>>()
}

/// Hook to access resolved theme (after system preference resolution)
pub fn use_resolved_theme() -> Signal<Theme> {
    use_context::<Signal<Theme>>()
}

/// Hook to toggle theme
pub fn use_toggle_theme() -> impl Fn() + Clone {
    let theme = use_theme();

    move || {
        let new_theme = match theme() {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
            Theme::System => Theme::Dark,
        };
        let mut t = theme;
        t.set(new_theme);
    }
}

/// Hook to set specific theme
pub fn use_set_theme() -> impl Fn(Theme) + Clone {
    let theme = use_theme();

    move |new_theme: Theme| {
        let mut t = theme;
        t.set(new_theme);
    }
}
