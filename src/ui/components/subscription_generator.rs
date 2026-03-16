//! Subscription Generator Component
//!
//! Builds encrypted subscription links masquerading as normal traffic.

use crate::ui::components::forms::{TextArea, TextInput};
use crate::ui::server_fns::generate_subscription_link;
use dioxus::prelude::*;

#[component]
pub fn SubscriptionGenerator() -> Element {
    let mut nodes_input = use_signal(|| String::new());
    let mut generated_link = use_signal(|| String::new());

    let handle_generate = move |_| async move {
        let nodes: Vec<String> = nodes_input.read().lines().map(String::from).collect();
        if let Ok(link) = generate_subscription_link(nodes).await {
            generated_link.set(link);
        }
    };

    rsx! {
        div { class: "p-4 bg-bg-secondary rounded-lg border border-border space-y-4",
            h3 { class: "text-sm font-bold text-gray-300 uppercase tracking-wider", "Subscription Builder" }

            TextArea {
                label: Some("Node Links (One per line)".to_string()),
                value: nodes_input,
                rows: 5,
                placeholder: Some("vless://...\nflowj://...".to_string()),
            }

            button {
                class: "w-full py-2 bg-primary text-white rounded font-medium hover:bg-primary-hover transition-colors",
                onclick: handle_generate,
                "Generate Decoy Link"
            }

            if !generated_link().is_empty() {
                div { class: "space-y-1 animate-fade-in",
                    label { class: "text-xs text-green-400 font-bold", "Secure Subscription URL" }
                    div { class: "p-3 bg-black/30 rounded border border-green-500/30 text-xs font-mono break-all text-gray-300 select-all",
                        "{generated_link}"
                    }
                    p { class: "text-[10px] text-gray-500",
                        "This link mimics a CDN content feed. Safe to share via unencrypted channels."
                    }
                }
            }
        }
    }
}
