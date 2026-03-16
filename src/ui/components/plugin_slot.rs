// src/ui/components/plugin_slot.rs
//! Plugin UI Slot Component
//!
//! Dynamically injects plugin UI components

use crate::domain::plugin_api::{UiPosition, UiSlot};
use dioxus::prelude::*;
use std::collections::HashMap;

/// Props for plugin slot
#[derive(Props, Clone, PartialEq)]
pub struct PluginSlotProps {
    /// Position to render plugins for
    pub position: UiPosition,
}

/// Plugin slot component
///
/// Renders all plugins registered for a specific UI position
#[component]
pub fn PluginSlot(props: PluginSlotProps) -> Element {
    let mut plugin_slots = use_signal(|| Vec::<UiSlot>::new());

    // Load plugin slots on mount
    use_effect(move || {
        spawn(async move {
            // In production, this would fetch from sidecar manager
            // For now, return empty list
            plugin_slots.set(Vec::new());
        });
    });

    let slots_for_position: Vec<UiSlot> = plugin_slots()
        .into_iter()
        .filter(|slot| slot.position == props.position)
        .collect();

    if slots_for_position.is_empty() {
        return rsx! { div { class: "plugin-slot-empty" } };
    }

    rsx! {
        div { class: "plugin-slot-container",
            for slot in slots_for_position {
                PluginSlotItem { slot: slot }
            }
        }
    }
}

/// Individual plugin slot item
#[component]
fn PluginSlotItem(slot: UiSlot) -> Element {
    let mut is_loaded = use_signal(|| false);
    let mut has_error = use_signal(|| false);

    // Load plugin component
    use_effect(move || {
        spawn(async move {
            // In production, this would load the component from component_url
            // For now, just mark as loaded
            #[cfg(feature = "web")]
            {
                gloo_timers::future::TimeoutFuture::new(100).await;
            }
            is_loaded.set(true);
        });
    });

    rsx! {
        div {
            class: "plugin-slot-item",
            "data-slot-id": "{slot.slot_id}",

            if !is_loaded() {
                div { class: "plugin-loading",
                    div { class: "loading-shimmer" }
                    span { "Loading plugin..." }
                }
            } else if has_error() {
                div { class: "plugin-error",
                    i { class: "fas fa-exclamation-triangle" }
                    span { "Plugin failed to load" }
                }
            } else {
                // Plugin component would be rendered here
                div { class: "plugin-content",
                    div { class: "plugin-placeholder",
                        i { class: "fas fa-puzzle-piece" }
                        h4 { "Plugin: {slot.slot_id}" }
                        p { "Component URL: {slot.component_url}" }
                    }
                }
            }
        }
    }
}

/// Plugin registry component for settings
#[component]
pub fn PluginRegistry() -> Element {
    let plugins = use_signal(|| Vec::<PluginRegistryItem>::new());
    let mut selected_plugin = use_signal(|| None::<String>);

    use crate::domain::plugin_api::PluginRegistryEntry;

    rsx! {
        div { class: "plugin-registry card",
            div { class: "plugin-registry-header",
                h3 { "Plugin Registry" }
                button {
                    class: "btn btn-primary",
                    i { class: "fas fa-plus" }
                    " Install Plugin"
                }
            }

            div { class: "plugin-list",
                if plugins().is_empty() {
                    div { class: "empty-state",
                        i { class: "fas fa-puzzle-piece" }
                        p { "No plugins installed" }
                        p { class: "text-muted", "Install plugins to extend functionality" }
                    }
                } else {
                    for plugin in plugins() {
                        PluginCard {
                            plugin: plugin,
                            on_select: move |id: String| {
                                selected_plugin.set(Some(id));
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, PartialEq)]
struct PluginRegistryItem {
    id: String,
    name: String,
    version: String,
    status: String,
}

/// Plugin card component
#[component]
fn PluginCard(plugin: PluginRegistryItem, on_select: EventHandler<String>) -> Element {
    let status_class = match plugin.status.as_str() {
        "running" => "status-success",
        "stopped" => "status-warning",
        "failed" => "status-error",
        _ => "status-muted",
    };

    rsx! {
        div {
            class: "plugin-card",
            onclick: move |_| on_select.call(plugin.id.clone()),

            div { class: "plugin-card-header",
                h4 { "{plugin.name}" }
                span { class: "plugin-version", "v{plugin.version}" }
            }

            div { class: "plugin-card-footer",
                span { class: "plugin-status {status_class}",
                    "● {plugin.status}"
                }
                div { class: "plugin-actions",
                    button { class: "btn-icon", title: "Configure",
                        i { class: "fas fa-cog" }
                    }
                    button { class: "btn-icon", title: "Stop",
                        i { class: "fas fa-stop" }
                    }
                }
            }
        }
    }
}
