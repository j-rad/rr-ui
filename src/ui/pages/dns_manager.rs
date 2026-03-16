//! DNS Integrity & Manager
//!
//! Visualization of DNS integrity checks and resolver management.

use crate::domain::models::DnsResolverStatus;
use crate::ui::components::card::Card;
use crate::ui::server_fns::get_dns_integrity;
use dioxus::prelude::*;

#[component]
pub fn DnsManagerPage() -> Element {
    let mut resolvers = use_signal(|| Vec::<DnsResolverStatus>::new());

    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        if let Ok(res) = get_dns_integrity().await {
            resolvers.set(res);
        }
    });

    rsx! {
        div { class: "p-6 space-y-6 animate-fade-in",
            div { class: "flex items-center justify-between",
                h1 { class: "text-2xl font-bold bg-gradient-to-r from-text-main to-text-secondary bg-clip-text text-transparent",
                    "DNS Integrity Monitor"
                }
                button {
                    class: "px-4 py-2 bg-bg-tertiary border border-border rounded text-sm hover:text-white transition-colors",
                    onclick: move |_| async move {
                        if let Ok(res) = get_dns_integrity().await {
                            resolvers.set(res);
                        }
                    },
                    "Refresh Checks"
                }
            }

            Card {
                title: "Resolver Health Map".to_string(),
                div { class: "p-4 grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4",
                    for res in resolvers() {
                        div {
                            class: "relative p-4 rounded-lg border flex flex-col gap-2 overflow-hidden transition-all hover:scale-[1.02]",
                            class: if res.is_poisoned {
                                "bg-red-500/10 border-red-500/30"
                            } else {
                                "bg-green-500/10 border-green-500/30"
                            },

                            // Header
                            div { class: "flex items-center justify-between",
                                div { class: "font-mono text-lg font-bold text-gray-200", "{res.resolver_ip}" }
                                if res.is_poisoned {
                                    span { class: "px-2 py-0.5 bg-red-500/20 text-red-400 text-xs font-bold rounded uppercase", "Poisoned" }
                                } else {
                                    span { class: "px-2 py-0.5 bg-green-500/20 text-green-400 text-xs font-bold rounded uppercase", "Verified" }
                                }
                            }

                            // Details
                            div { class: "space-y-1 text-sm text-gray-400",
                                div { class: "flex justify-between",
                                    span { "Latency:" }
                                    span { class: if res.latency_ms < 20 { "text-green-400" } else { "text-yellow-400" }, "{res.latency_ms} ms" }
                                }
                                div { class: "flex justify-between",
                                    span { "Hash:" }
                                    span { class: "font-mono text-xs truncate w-24 text-right", "{res.query_hash}" }
                                }
                            }

                            // Background graphic
                            div { class: "absolute -bottom-4 -right-4 text-[100px] opacity-5 pointer-events-none select-none",
                                if res.is_poisoned { "💀" } else { "🛡️" }
                            }
                        }
                    }
                }
            }
        }
    }
}
