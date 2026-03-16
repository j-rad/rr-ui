//! Dioxus App Entry Point
//!
//! Main application component with router and layout.

use dioxus::prelude::*;

use super::components::command_palette::CommandPalette;
use super::components::sidebar::Sidebar;
use super::components::toast::ToastContainer;
use super::pages::{
    backups::BackupsPage, connections::ConnectionsPage, dashboard::DashboardPage,
    diagnostics::DiagnosticsPage, forms_demo::FormsDemoPage, inbounds::InboundsPage,
    login::LoginPage, logs::LogsPage, mesh::MeshPage, migration::MigrationPage,
    rustray::RustRayPage, settings::SettingsPage,
};
use super::state::GlobalState;

/// Main application component
#[component]
pub fn App() -> Element {
    // Initialize global state
    let state = use_context_provider(|| GlobalState::new());

    // Provide toast store specifically if components ask for it directly
    use_context_provider(|| state.toast.clone());

    // Start background sync
    use_hook(|| {
        state.init_sync();
    });

    rsx! {
        // Tailwind Configuration (Must be defined BEFORE loading Tailwind)
        script {
            "tailwind.config = {{
                darkMode: 'class',
                theme: {{
                    extend: {{
                        colors: {{
                            primary: '#00d4ff',
                            'primary-hover': '#22e5ff',
                            'primary-active': '#00b8e0',
                            bg: {{
                                DEFAULT: '#030712',
                                panel: '#0a0f1a',
                                header: '#0a0f1a',
                                card: 'rgba(17, 24, 39, 0.75)',
                                tertiary: '#111827',
                            }},
                            border: {{
                                DEFAULT: 'rgba(71, 85, 105, 0.25)',
                                light: 'rgba(51, 65, 85, 0.15)',
                            }},
                            text: {{
                                main: 'rgba(248, 250, 252, 0.92)',
                                secondary: 'rgba(148, 163, 184, 0.85)',
                                muted: 'rgba(100, 116, 139, 0.7)'
                            }},
                            glass: {{
                                bg: 'rgba(17, 24, 39, 0.65)',
                                border: 'rgba(148, 163, 184, 0.08)',
                            }},
                            slate: {{
                                700: '#334155',
                                800: '#1E293B',
                                900: '#0F172A',
                            }},
                            cyan: {{
                                400: '#22e5ff',
                                500: '#00d4ff',
                                600: '#00b8e0',
                            }},
                            accent: {{
                                purple: '#c084fc',
                                pink: '#f472b6',
                                orange: '#fb923c',
                            }}
                        }},
                        fontFamily: {{
                            sans: ['Inter', 'system-ui', 'sans-serif'],
                        }},
                        animation: {{
                            'fade-in': 'fadeIn 0.4s ease-out',
                            'slide-up': 'slideUp 0.3s cubic-bezier(0.16, 1, 0.3, 1)',
                            'slide-in': 'slideInRight 0.3s ease-out',
                            'shimmer': 'shimmer 2s infinite',
                            'glow': 'glowPulse 2s ease-in-out infinite',
                            'pulse-glow': 'pulseGlow 2s ease-in-out infinite',
                            'float': 'float 3s ease-in-out infinite',
                            'counter': 'counterUp 0.4s ease-out',
                        }},
                        keyframes: {{
                            fadeIn: {{
                                '0%': {{ opacity: '0', transform: 'translateY(8px)' }},
                                '100%': {{ opacity: '1', transform: 'translateY(0)' }},
                            }},
                            slideUp: {{
                                '0%': {{ opacity: '0', transform: 'translateY(24px) scale(0.96)' }},
                                '100%': {{ opacity: '1', transform: 'translateY(0) scale(1)' }},
                            }},
                            slideInRight: {{
                                '0%': {{ opacity: '0', transform: 'translateX(24px)' }},
                                '100%': {{ opacity: '1', transform: 'translateX(0)' }},
                            }},
                            shimmer: {{
                                '0%': {{ transform: 'translateX(-100%)' }},
                                '100%': {{ transform: 'translateX(100%)' }},
                            }},
                            glowPulse: {{
                                '0%, 100%': {{ boxShadow: '0 0 8px rgba(0, 212, 255, 0.3)' }},
                                '50%': {{ boxShadow: '0 0 24px rgba(0, 212, 255, 0.5), 0 0 40px rgba(0, 212, 255, 0.3)' }},
                            }},
                            pulseGlow: {{
                                '0%, 100%': {{ opacity: '1', transform: 'scale(1)' }},
                                '50%': {{ opacity: '0.7', transform: 'scale(1.05)' }},
                            }},
                            float: {{
                                '0%, 100%': {{ transform: 'translateY(0)' }},
                                '50%': {{ transform: 'translateY(-6px)' }},
                            }},
                            counterUp: {{
                                '0%': {{ opacity: '0', transform: 'translateY(8px)' }},
                                '100%': {{ opacity: '1', transform: 'translateY(0)' }},
                            }}
                        }},
                        backdropBlur: {{
                            xs: '2px',
                            xl: '20px',
                        }},
                        boxShadow: {{
                            'glow': '0 0 20px rgba(0, 212, 255, 0.35)',
                            'glow-lg': '0 0 40px rgba(0, 212, 255, 0.4)',
                        }}
                    }},
                }},
            }};"
        }
        // Tailwind CSS CDN
        script { src: "https://cdn.tailwindcss.com?plugins=forms,container-queries" }
        // Google Fonts
        link { href: "https://fonts.googleapis.com", rel: "preconnect" }
        link { href: "https://fonts.gstatic.com", rel: "preconnect", crossorigin: "true" }
        link { href: "https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&display=swap", rel: "stylesheet" }
        link { href: "https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&display=swap", rel: "stylesheet" }

        Router::<Route> {}
    }
}

/// Application routes
#[derive(Clone, Routable, Debug, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(PanelLayout)]
        #[route("/panel")]
        Dashboard {},
        #[route("/panel/inbounds")]
        Inbounds {},
        #[route("/panel/connections")]
        Connections {},
        #[route("/panel/rustray")]
        RustRayConfig {},
        #[route("/panel/logs")]
        Logs {},
        #[route("/panel/backups")]
        Backups {},
        #[route("/panel/settings")]
        Settings {},
        #[route("/panel/forms-demo")]
        FormsDemo {},
        #[route("/panel/dns")]
        DnsManager {},
        #[route("/panel/mesh")]
        Mesh {},
        #[route("/panel/migration")]
        Migration {},
        #[route("/panel/diagnostics")]
        Diagnostics {},
    #[end_layout]
    
    #[route("/login")]
    Login {},
    
    #[route("/")]
    Home {},
}

/// Main panel layout with sidebar and auth check
#[component]
fn PanelLayout() -> Element {
    let state = use_context::<GlobalState>();
    let sidebar_collapsed = state.sidebar_collapsed;
    let theme = state.theme;
    let navigator = use_navigator();

    // Route Protection
    let is_authenticated = state.is_authenticated;
    use_effect(move || {
        if !*is_authenticated.read() {
            navigator.push(Route::Login {});
        }
    });

    if !*is_authenticated.read() {
        return rsx! { div {} }; // Return empty while redirecting
    }

    rsx! {
        div { class: "flex h-screen w-full bg-bg overflow-hidden text-text-main font-sans antialiased", "data-theme": "{theme}",
            Sidebar { collapsed: sidebar_collapsed }
            // Main Content Wrapper
            div { class: "flex-1 flex flex-col min-w-0 overflow-hidden relative",
                 // Scrollable Content Area
                div { class: "flex-1 overflow-y-auto",
                    div { class: "container mx-auto max-w-7xl",
                        Outlet::<Route> {}
                    }
                }
            }
            ToastContainer {}
            CommandPalette {}
        }
    }
}

/// Home redirect
#[component]
fn Home() -> Element {
    // Simple redirect to dashboard (which handles auth check)
    let nav = use_navigator();
    nav.push(Route::Dashboard {});
    rsx! {
        div { "Redirecting..." }
    }
}

/// Login page wrapper
#[component]
fn Login() -> Element {
    rsx! { LoginPage {} }
}

/// Dashboard page wrapper
#[component]
fn Dashboard() -> Element {
    rsx! { DashboardPage {} }
}

/// Inbounds page wrapper
#[component]
fn Inbounds() -> Element {
    rsx! { InboundsPage {} }
}

/// Connections page wrapper
#[component]
fn Connections() -> Element {
    rsx! { ConnectionsPage {} }
}

/// RustRay config page wrapper
#[component]
fn RustRayConfig() -> Element {
    rsx! { RustRayPage {} }
}

/// Logs page wrapper
#[component]
fn Logs() -> Element {
    rsx! { LogsPage {} }
}

/// Backups page wrapper
#[component]
fn Backups() -> Element {
    rsx! { BackupsPage {} }
}

/// Settings page wrapper
#[component]
fn Settings() -> Element {
    rsx! { SettingsPage {} }
}

/// Forms demo page wrapper
#[component]
fn FormsDemo() -> Element {
    rsx! { FormsDemoPage {} }
}

/// DNS Manager page wrapper
fn DnsManager() -> Element {
    rsx! { crate::ui::pages::dns_manager::DnsManagerPage {} }
}

/// Mesh Dashboard page wrapper
#[component]
fn Mesh() -> Element {
    rsx! { MeshPage {} }
}

/// Migration Wizard page wrapper
#[component]
fn Migration() -> Element {
    rsx! { MigrationPage {} }
}

/// Diagnostics page wrapper
#[component]
fn Diagnostics() -> Element {
    rsx! { DiagnosticsPage {} }
}
