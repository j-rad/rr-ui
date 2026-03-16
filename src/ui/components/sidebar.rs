//! Sidebar Component
//!
//! Navigation sidebar matching the Svelte Sidebar.svelte component.

use dioxus::prelude::*;

use crate::ui::app::Route;
use crate::ui::state::GlobalState;

/// Menu item definition
struct MenuItem {
    route: Route,
    label: &'static str,
    icon: &'static str,
}

// Helper to get menu items dynamically if needed, but static is fine for now
fn get_menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem {
            route: Route::Dashboard {},
            label: "Overview",
            icon: "dashboard",
        },
        MenuItem {
            route: Route::Inbounds {},
            label: "Inbounds",
            icon: "input",
        },
        MenuItem {
            route: Route::Connections {},
            label: "Connections",
            icon: "monitoring",
        },
        MenuItem {
            route: Route::RustRayConfig {},
            label: "RustRay Config",
            icon: "settings_ethernet",
        },
        MenuItem {
            route: Route::Logs {},
            label: "Logs",
            icon: "assignment",
        },
        MenuItem {
            route: Route::Backups {},
            label: "Backups",
            icon: "database",
        },
        MenuItem {
            route: Route::Settings {},
            label: "Settings",
            icon: "settings",
        },
        MenuItem {
            route: Route::Diagnostics {},
            label: "Diagnostics",
            icon: "speed",
        },
        MenuItem {
            route: Route::Mesh {},
            label: "Mesh Cluster",
            icon: "hub",
        },
        MenuItem {
            route: Route::Migration {},
            label: "Migration",
            icon: "upload_file",
        },
    ]
}

#[derive(Props, Clone, PartialEq)]
pub struct SidebarProps {
    pub collapsed: Signal<bool>,
}

#[component]
pub fn Sidebar(props: SidebarProps) -> Element {
    let mut collapsed = props.collapsed;
    let state = use_context::<GlobalState>();
    let theme = state.theme;
    let current_route = use_route::<Route>();

    let is_dark = theme().starts_with("dark") || theme() == "ultra-dark";

    let width_class = if collapsed() {
        "w-20"
    } else {
        "w-[280px]" // Slightly wider to match 3x-ui
    };

    let toggle_theme = move |_| {
        let mut t = theme;
        t.set(if t() == "dark" { "light" } else { "dark" }.to_string());
    };

    let toggle_sidebar = move |_| {
        collapsed.set(!collapsed());
    };

    let logout = {
        let mut is_authenticated = state.is_authenticated;
        let mut auth_token = state.auth_token;
        move |_| {
            // Clear global state
            is_authenticated.set(false);
            auth_token.set(None);

            // Redirect to login
            use_navigator().push(crate::ui::app::Route::Login {});
        }
    };

    let menu_items = get_menu_items();

    rsx! {
        aside { class: "h-screen bg-gradient-to-b from-bg-secondary to-bg border-r border-border-light/30 flex flex-col shrink-0 transition-all duration-300 shadow-xl {width_class}",
            // Logo
            div { class: "h-16 flex items-center px-6 border-b border-border-light/30 overflow-hidden whitespace-nowrap",
                div { class: "flex items-center gap-3 font-bold text-lg tracking-tight text-text-main",
                    if collapsed() {
                         span { class: "material-symbols-outlined text-primary text-2xl drop-shadow-[0_0_8px_rgba(0,212,255,0.4)]", "token" }
                    } else {
                        span { class: "material-symbols-outlined text-primary text-2xl drop-shadow-[0_0_8px_rgba(0,212,255,0.4)]", "token" }
                        span { class: "bg-gradient-to-r from-primary to-cyan-400 bg-clip-text text-transparent", "RR-UI" }
                    }
                }
            }

            // Navigation menu
            nav { class: "flex-1 overflow-y-auto py-6 px-3 space-y-1.5",
                for item in menu_items {
                    {render_menu_item(&item, collapsed(), &current_route)}
                }
            }

            // Footer actions
            div { class: "p-4 border-t border-border-light/30 space-y-1",
                // Theme toggle
                button {
                    class: "flex items-center w-full px-3 py-2.5 text-sm font-medium text-text-secondary rounded-lg hover:bg-white/5 hover:text-text-main transition-all duration-200 group",
                    onclick: toggle_theme,
                    if is_dark {
                        span { class: "material-symbols-outlined text-[20px] mr-3 text-text-muted group-hover:text-amber-400 transition-colors", "light_mode" }
                    } else {
                        span { class: "material-symbols-outlined text-[20px] mr-3 text-text-muted group-hover:text-indigo-400 transition-colors", "dark_mode" }
                    }
                    if !collapsed() {
                        span { "Theme" }
                    }
                }

                // Collapse toggle
                button {
                    class: "flex items-center w-full px-3 py-2.5 text-sm font-medium text-text-secondary rounded-lg hover:bg-white/5 hover:text-text-main transition-all duration-200 group",
                    onclick: toggle_sidebar,
                    if collapsed() {
                        span { class: "material-symbols-outlined text-[20px] mr-3 text-text-muted group-hover:text-primary transition-colors", "chevron_right" }
                    } else {
                        span { class: "material-symbols-outlined text-[20px] mr-3 text-text-muted group-hover:text-primary transition-colors", "chevron_left" }
                        span { "Collapse" }
                    }
                }

                // Logout
                button {
                    class: "flex items-center w-full px-3 py-2.5 text-sm font-medium text-text-secondary rounded-lg hover:bg-rose-500/10 hover:text-rose-400 transition-all duration-200 group",
                    onclick: logout,
                    span { class: "material-symbols-outlined text-[20px] mr-3 text-text-muted group-hover:text-rose-500 transition-colors", "logout" }
                    if !collapsed() {
                        span { "Log Out" }
                    }
                }
            }
        }
    }
}

fn render_menu_item(item: &MenuItem, collapsed: bool, current_route: &Route) -> Element {
    // Simple equality check for active route
    let is_active = std::mem::discriminant(&item.route) == std::mem::discriminant(current_route);

    let mut class =
        "flex items-center px-3 py-2.5 text-sm font-medium rounded-lg transition-all duration-200 group "
            .to_string();
    if is_active {
        class.push_str(
            "text-white bg-gradient-to-r from-primary to-cyan-400 shadow-lg shadow-primary/25",
        );
    } else {
        class.push_str("text-text-secondary hover:bg-white/5 hover:text-text-main");
    }

    let icon_class = if is_active {
        "material-symbols-outlined text-[20px] mr-3 drop-shadow-[0_0_4px_rgba(255,255,255,0.3)]"
    } else {
        "material-symbols-outlined text-[20px] mr-3 text-text-muted group-hover:text-primary transition-colors"
    };

    rsx! {
        Link {
            to: item.route.clone(),
            class: "{class}",
            div { class: "{icon_class}",
                 "{item.icon}"
            }
            if !collapsed {
                span { "{item.label}" }
            }
        }
    }
}
