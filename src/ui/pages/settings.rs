//! Settings Page

use crate::domain::models::{AssetStatus, SystemHealth};
use crate::ui::components::card::Card;
use crate::ui::components::forms::{NumberInput, Switch, TextInput};
use crate::ui::server_fns::{
    control_core_process, get_asset_status, get_system_health, reload_routing_assets,
};
use dioxus::prelude::*;

#[component]
pub fn SettingsPage() -> Element {
    // Form state
    let mut panel_port = use_signal(|| 2053i64);
    let mut username = use_signal(|| String::from("admin"));
    let mut session_timeout = use_signal(|| 30i64);
    let mut ssl_enabled = use_signal(|| false);
    let mut auto_backup = use_signal(|| true);
    let mut traffic_reset_day = use_signal(|| 1i64);

    // Lifecycle State
    let mut health = use_signal(|| SystemHealth::default());
    let mut geoip_status = use_signal(|| AssetStatus::default());
    let mut geosite_status = use_signal(|| AssetStatus::default());
    let mut core_loading = use_signal(|| false);

    // Fetch settings on mount
    let settings_resource =
        use_resource(move || async move { crate::ui::server_fns::get_panel_settings().await });

    // Fetch Health & Assets
    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        loop {
            if let Ok(h) = get_system_health().await {
                health.set(h);
            }
            if let Ok(ip) = get_asset_status("geoip.dat".to_string()).await {
                geoip_status.set(ip);
            }
            if let Ok(site) = get_asset_status("geosite.dat".to_string()).await {
                geosite_status.set(site);
            }
            crate::ui::sleep::sleep(5000).await;
        }
    });

    // Update signals when settings are loaded
    use_effect(move || {
        if let Some(res) = settings_resource.value().as_ref() {
            if let Ok(settings) = &*res {
                panel_port.set(settings.web_port as i64);
                username.set(settings.username.clone());
            }
        }
    });

    // Save Handler
    let handle_save = move |_| {
        spawn(async move {
            if let Some(res) = settings_resource.value().as_ref() {
                if let Ok(current_settings) = &*res {
                    let mut new_settings = current_settings.clone();
                    new_settings.web_port = panel_port() as u16;
                    new_settings.username = username();
                    match crate::ui::server_fns::update_panel_settings(new_settings).await {
                        Ok(_) => log::info!("Settings saved successfully"),
                        Err(e) => log::error!("Failed to save settings: {}", e),
                    }
                }
            }
        });
    };

    let handle_core_action = move |action: String| async move {
        core_loading.set(true);
        let _ = control_core_process(action).await;
        core_loading.set(false);
    };

    let handle_update_assets = move |_| async move {
        core_loading.set(true);
        let _ = reload_routing_assets().await;
        // Refresh statuses
        if let Ok(ip) = get_asset_status("geoip.dat".to_string()).await {
            geoip_status.set(ip);
        }
        if let Ok(site) = get_asset_status("geosite.dat".to_string()).await {
            geosite_status.set(site);
        }
        core_loading.set(false);
    };

    rsx! {
        div { class: "p-6 space-y-6 animate-fade-in",
            h1 { class: "text-2xl font-bold bg-gradient-to-r from-text-main to-text-secondary bg-clip-text text-transparent", "System Control" }

            // 1. Asset Status Dashboard
            div { class: "grid grid-cols-1 lg:grid-cols-2 gap-6",
                Card {
                    title: "Asset Status".to_string(),
                    actions: rsx! {
                        button {
                            class: "px-3 py-1 bg-primary text-white text-xs rounded hover:bg-primary-hover disabled:opacity-50",
                            onclick: handle_update_assets,
                            disabled: core_loading,
                            if core_loading() { "Updating..." } else { "Check for Updates" }
                        }
                    },
                    div { class: "space-y-4",
                        // GeoIP
                        div { class: "flex items-center justify-between p-3 bg-white/5 rounded border border-white/5",
                            div { class: "flex items-center gap-3",
                                span { class: "material-symbols-outlined text-blue-400", "public" }
                                div {
                                    div { class: "text-sm font-medium text-gray-200", "GeoIP Database" }
                                    div { class: "text-xs text-gray-500 font-mono", "{geoip_status.read().hash.get(0..8).unwrap_or(\"Unknown\")}" }
                                }
                            }
                            div { class: "text-right",
                                div { class: "text-xs text-gray-400", "Version" }
                                div { class: "text-sm font-bold text-white", "{geoip_status.read().version}" }
                            }
                        }
                        // GeoSite
                        div { class: "flex items-center justify-between p-3 bg-white/5 rounded border border-white/5",
                            div { class: "flex items-center gap-3",
                                span { class: "material-symbols-outlined text-purple-400", "dns" }
                                div {
                                    div { class: "text-sm font-medium text-gray-200", "GeoSite Database" }
                                    div { class: "text-xs text-gray-500 font-mono", "{geosite_status.read().hash.get(0..8).unwrap_or(\"Unknown\")}" }
                                }
                            }
                            div { class: "text-right",
                                div { class: "text-xs text-gray-400", "Version" }
                                div { class: "text-sm font-bold text-white", "{geosite_status.read().version}" }
                            }
                        }
                    }
                }

                // 2. Core Process Supervisor
                Card {
                    title: "Core Supervisor".to_string(),
                    div { class: "space-y-6",
                        // Metrics
                        div { class: "grid grid-cols-2 gap-4",
                            div { class: "p-3 bg-black/20 rounded text-center",
                                div { class: "text-xs text-gray-500 uppercase", "Open Sockets" }
                                div { class: "text-xl font-mono font-bold text-blue-400", "{health.read().open_sockets}" }
                            }
                            div { class: "p-3 bg-black/20 rounded text-center",
                                div { class: "text-xs text-gray-500 uppercase", "Threads" }
                                div { class: "text-xl font-mono font-bold text-green-400", "{health.read().thread_count}" }
                            }
                            div { class: "p-3 bg-black/20 rounded text-center",
                                div { class: "text-xs text-gray-500 uppercase", "Uptime" }
                                div { class: "text-xl font-mono font-bold text-white", "{health.read().uptime_seconds / 60}m" }
                            }
                            div { class: "p-3 bg-black/20 rounded text-center",
                                div { class: "text-xs text-gray-500 uppercase", "Memory" }
                                div { class: "text-xl font-mono font-bold text-yellow-400", "{health.read().memory_usage_mb} MB" }
                            }
                        }

                        // Controls
                        div { class: "grid grid-cols-2 gap-3",
                            button {
                                class: "py-2 bg-green-500/10 text-green-400 border border-green-500/20 rounded hover:bg-green-500/20 transition",
                                onclick: move |_| handle_core_action("start".to_string()),
                                "Start Core"
                            }
                            button {
                                class: "py-2 bg-red-500/10 text-red-400 border border-red-500/20 rounded hover:bg-red-500/20 transition",
                                onclick: move |_| handle_core_action("stop".to_string()),
                                "Stop Core"
                            }
                            button {
                                class: "py-2 bg-blue-500/10 text-blue-400 border border-blue-500/20 rounded hover:bg-blue-500/20 transition",
                                onclick: move |_| handle_core_action("restart".to_string()),
                                "Restart"
                            }
                            button {
                                class: "py-2 bg-yellow-500/10 text-yellow-400 border border-yellow-500/20 rounded hover:bg-yellow-500/20 transition",
                                onclick: move |_| handle_core_action("reload".to_string()),
                                "Hot Reload"
                            }
                        }
                    }
                }
            }

            // 3. General Settings (Original)
            div { class: "grid grid-cols-1 lg:grid-cols-2 gap-6",
                Card {
                    title: "General".to_string(),
                    actions: rsx! {
                        button {
                            class: "px-4 py-2 bg-primary hover:bg-primary-hover text-white text-sm font-medium rounded transition shadow-sm",
                            onclick: handle_save,
                            "Save Changes"
                        }
                    },
                    div { class: "space-y-4",
                        NumberInput {
                            label: Some("Panel Port".to_string()),
                            value: panel_port,
                            min: Some(1024i64),
                            max: Some(65535i64),
                            required: true,
                        }
                        TextInput {
                            label: Some("Username".to_string()),
                            value: username,
                            required: true,
                        }
                    }
                }

                // Security
                Card {
                     title: "Security".to_string(),
                     div { class: "space-y-6",
                        Switch { label: Some("Enable SSL/TLS".to_string()), value: ssl_enabled }
                        Switch { label: Some("Auto Backup".to_string()), value: auto_backup }
                    }
                }
            }
        }
    }
}
