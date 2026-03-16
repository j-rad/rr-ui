use crate::models::TunConfig;
use crate::ui::components::badge::{Badge, BadgeVariant};
use crate::ui::components::{
    button::{Button, ButtonVariant},
    card::{Card, CardContent, CardHeader, CardTitle},
    input::{Input, InputType},
};
#[cfg(feature = "web")]
use crate::ui::server_fns::{get_tun_config, set_tun_config};
use dioxus::prelude::*;

#[component]
pub fn TunDashboard() -> Element {
    let mut tun_config = use_signal(|| TunConfig::default());
    let mut is_loading = use_signal(|| true);
    let mut error_msg = use_signal(|| Option::<String>::None);
    let mut success_msg = use_signal(|| Option::<String>::None);

    // Fetch config on mount
    use_effect(move || {
        to_owned![tun_config, is_loading, error_msg];
        spawn(async move {
            #[cfg(feature = "web")]
            {
                match get_tun_config().await {
                    Ok(config) => {
                        tun_config.set(config);
                        is_loading.set(false);
                    }
                    Err(e) => {
                        error_msg.set(Some(format!("Failed to load TUN config: {}", e)));
                        is_loading.set(false);
                    }
                }
            }
            #[cfg(not(feature = "web"))]
            {
                is_loading.set(false);
            }
        });
    });

    let handle_save = move |_| {
        to_owned![tun_config, error_msg, success_msg];
        spawn(async move {
            // Convert signal to owned config for sending
            // Note: In real app, we might need to deep clone or reconstruct if TunConfig has lifetimes
            // Here assuming we can just clone the data structure if it owns its data or we convert it.
            // The TunConfig in models has lifetimes, but server fn returns static.
            // We need to ensure we send back correct structure.
            let config = tun_config.read().clone();

            #[cfg(feature = "web")]
            {
                match set_tun_config(config).await {
                    Ok(_) => {
                        success_msg.set(Some("Configuration saved successfully".to_string()));
                        error_msg.set(None);
                        // Clear success message after 3 seconds
                        spawn(async move {
                            crate::ui::sleep::sleep(3000 as u64).await;
                            success_msg.set(None);
                        });
                    }
                    Err(e) => {
                        error_msg.set(Some(format!("Failed to save config: {}", e)));
                        success_msg.set(None);
                    }
                }
            }
        });
    };

    if *is_loading.read() {
        return rsx! {
            div { class: "flex justify-center p-8",
                div { class: "animate-spin rounded-full h-8 w-8 border-b-2 border-primary" }
            }
        };
    }

    let config = tun_config.read();

    let tun_variant = if config.enable {
        BadgeVariant::Success
    } else {
        BadgeVariant::Neutral
    };
    let tun_text = if config.enable {
        "TUN Active"
    } else {
        "TUN Disabled"
    };

    rsx! {
        div { class: "space-y-6",
            // Header Section
            div { class: "flex items-center justify-between mb-6",
                div {
                    h2 { class: "text-xl font-bold text-white", "Kernel TUN Interface" }
                    p { class: "text-sm text-gray-400", "Manage routing and network interface settings" }
                }
                Badge {
                    variant: tun_variant,
                    "{tun_text}"
                }
            }

            if let Some(msg) = error_msg.read().as_ref() {
                div { class: "bg-red-500/10 border border-red-500/20 text-red-500 p-4 rounded-lg", "{msg}" }
            }

            if let Some(msg) = success_msg.read().as_ref() {
                div { class: "bg-green-500/10 border border-green-500/20 text-green-500 p-4 rounded-lg", "{msg}" }
            }

            // Main Configuration Card
            Card {
                CardHeader {
                    CardTitle { "TUN Interface Settings" }
                }
                CardContent {
                    div { class: "grid grid-cols-1 md:grid-cols-2 gap-6",

                        // Enable Toggle
                        div { class: "col-span-1 md:col-span-2 flex items-center justify-between p-4 bg-bg-panel rounded-lg border border-border",
                            div {
                                h3 { class: "font-medium text-white", "Enable TUN Mode" }
                                p { class: "text-sm text-gray-400", "Redirect system traffic through the core" }
                            }
                            // Using a simple checkbox for now, replace with Switch component if available
                            input {
                                "type": "checkbox",
                                class: "w-6 h-6 rounded border-gray-600 text-primary focus:ring-primary bg-bg",
                                checked: config.enable,
                                onchange: move |e| {
                                    tun_config.write().enable = e.value() == "true";
                                }
                            }
                        }

                        // Interface Name
                        div {
                            label { class: "block text-sm font-medium text-gray-400 mb-1", "Interface Name" }
                            Input {
                                value: config.interface.to_string(),
                                oninput: move |e: String| {
                                    tun_config.write().interface = std::borrow::Cow::Owned(e);
                                },
                                placeholder: "tun0",
                            }
                        }

                        // MTU
                        div {
                            label { class: "block text-sm font-medium text-gray-400 mb-1", "MTU" }
                            Input {
                                r#type: InputType::Number,
                                value: config.mtu.to_string(),
                                oninput: move |e: String| {
                                    if let Ok(val) = e.parse::<u32>() {
                                        tun_config.write().mtu = val;
                                    }
                                },
                                placeholder: "9000",
                            }
                        }

                        // Stack Selection
                        div {
                            label { class: "block text-sm font-medium text-gray-400 mb-1", "Stack" }
                             select {
                                class: "w-full bg-bg border border-border rounded px-3 py-2 text-white focus:border-primary focus:ring-1 focus:ring-primary",
                                value: config.stack.to_string(),
                                onchange: move |e: Event<FormData>| {
                                    tun_config.write().stack = std::borrow::Cow::Owned(e.value());
                                },
                                option { value: "gvisor", "gVisor (User-space)" }
                                option { value: "system", "System (Kernel-space)" }
                                option { value: "mixed", "Mixed" }
                            }
                        }

                        // Strict Route Toggle
                        div { class: "flex items-center justify-between p-3 bg-bg-panel rounded border border-border",
                            span { class: "text-sm text-gray-300", "Strict Route" }
                            input {
                                "type": "checkbox",
                                class: "rounded border-gray-600 text-primary focus:ring-primary bg-bg",
                                checked: config.strict_route,
                                onchange: move |e| {
                                    tun_config.write().strict_route = e.value() == "true";
                                }
                            }
                        }

                         // Endpoint Independent NAT
                        div { class: "flex items-center justify-between p-3 bg-bg-panel rounded border border-border",
                            span { class: "text-sm text-gray-300", "Endpoint Independent NAT" }
                            input {
                                "type": "checkbox",
                                class: "rounded border-gray-600 text-primary focus:ring-primary bg-bg",
                                checked: config.endpoint_independent_nat,
                                onchange: move |e| {
                                    tun_config.write().endpoint_independent_nat = e.value() == "true";
                                }
                            }
                        }
                    }

                    // Route Addresses
                     div { class: "mt-6",
                        label { class: "block text-sm font-medium text-gray-400 mb-1", "Route Addresses (CIDR)" }
                        textarea {
                            class: "w-full bg-bg border border-border rounded px-3 py-2 text-white font-mono text-sm min-h-[100px] focus:border-primary focus:ring-1 focus:ring-primary",
                            value: config.route_address.join("\n"),
                            placeholder: "10.0.0.0/8\n172.16.0.0/12\n192.168.0.0/16",
                            oninput: move |e: Event<FormData>| {
                                let val = e.value();
                                let addresses: Vec<std::borrow::Cow<'static, str>> = val.lines()
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty())
                                    .map(std::borrow::Cow::Owned)
                                    .collect();
                                tun_config.write().route_address = addresses;
                            }
                        }
                         p { class: "text-xs text-gray-500 mt-1", "One CIDR per line. Standard private ranges are recommended." }
                    }

                     // Route Exclude Addresses
                     div { class: "mt-4",
                        label { class: "block text-sm font-medium text-gray-400 mb-1", "Exclude Addresses (CIDR)" }
                         textarea {
                            class: "w-full bg-bg border border-border rounded px-3 py-2 text-white font-mono text-sm min-h-[100px] focus:border-primary focus:ring-1 focus:ring-primary",
                            value: config.route_exclude_address.join("\n"),
                            placeholder: "127.0.0.1/32\n::1/128",
                            oninput: move |e: Event<FormData>| {
                                let val = e.value();
                                let addresses: Vec<std::borrow::Cow<'static, str>> = val.lines()
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty())
                                    .map(std::borrow::Cow::Owned)
                                    .collect();
                                tun_config.write().route_exclude_address = addresses;
                            }
                        }
                    }
                }
            }

            // Actions
            div { class: "flex justify-end gap-3",
                 Button {
                    variant: ButtonVariant::Primary,
                    onclick: handle_save,
                    "Save Configuration"
                }
            }
        }
    }
}
