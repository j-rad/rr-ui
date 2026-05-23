//! Dioxus App Entry Point
//!
//! Main application component with router and layout.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use crate::ui::state::UIState;
pub use crate::ui::error_bridge::TacticalMessage;
use crate::ui::components::hydration_gate::HydrationGate;
use crate::domain::proxy_core::CoreConfig;

/// Synchronization status for the UI
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum SyncStatus {
    Initial,
    Syncing,
    Live,
    Stale(TacticalMessage),
}

impl Default for SyncStatus {
    fn default() -> Self {
        Self::Initial
    }
}

/// Transition status to 'Live' ONLY when the core configuration is fully synced.
pub fn on_first_grpc_delta(mut status: Signal<SyncStatus>) {
    status.set(SyncStatus::Live);
}

use super::components::command_palette::CommandPalette;
use super::components::sidebar::Sidebar;
use super::components::toast::ToastContainer;
use super::pages::{
    backups::BackupsPage, connections::ConnectionsPage, dashboard::DashboardPage,
    diagnostics::DiagnosticsPage, forms_demo::FormsDemoPage, inbounds::InboundsPage,
    login::LoginPage, logs::LogsPage, mesh::MeshPage, migration::MigrationPage,
    rustray::RustRayPage, settings::SettingsPage, traffic_mimicry::TrafficMimicryPage,
};

/// Main application component
#[component]
pub fn App() -> Element {
    // Initialize global state
    let state = use_context_provider(|| UIState::new());

    // Provide toast store specifically if components ask for it directly
    use_context_provider(|| state.toast.clone());

    // Provide CoreConfig context as requested
    use_context_provider(|| Arc::new(RwLock::new(CoreConfig {
        log_level: crate::domain::proxy_core::LogLevel::Info,
        inbounds: vec![],
        outbounds: vec![],
        routing: crate::domain::proxy_core::RoutingConfig {
            rules: vec![],
            domain_strategy: crate::domain::proxy_core::DomainStrategy::AsIs,
        },
        dns: None,
    })));

    // Start background sync
    use_hook(|| {
        state.init_sync();
    });

    rsx! {
        // Local CSS and Embedded Tailwind
        style { "{crate::ui::theme::assets::LAYOUT_CSS}" }
        style { "{crate::ui::theme::assets::INTER_FONT_CSS}" }
        style { "{crate::ui::theme::assets::TAILWIND_CSS}" }
        // All fonts and styles served from binary — zero network requests

        HydrationGate {
            Router::<Route> {}
        }
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
        #[route("/panel/traffic-mimicry")]
        TrafficMimicry {},
    #[end_layout]
    
    #[route("/login")]
    Login {},
    
    #[route("/")]
    Home {},
}

/// Main panel layout with sidebar and auth check
#[component]
fn PanelLayout() -> Element {
    let state = use_context::<UIState>();
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

/// Traffic Mimicry page wrapper
#[component]
fn TrafficMimicry() -> Element {
    rsx! { TrafficMimicryPage {} }
}
