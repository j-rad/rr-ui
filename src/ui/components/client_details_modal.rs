//! Client Details Modal
//!
//! Comprehensive modal for client management:
//! - Lifecycle status and actions (enable/disable, reset traffic)
//! - Subscription generation (Clash, V2RayN, QR)
//! - Detailed traffic statistics

use crate::domain::models::{Client, Inbound};
use crate::ui::components::client_lifecycle::{ClientAction, ClientLifecycleCard};
use crate::ui::components::modal::Modal;
use crate::ui::components::subscription_generator::SubscriptionGenerator;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ClientDetailsModalProps {
    pub open: Signal<bool>,
    pub client: Option<Client>,
    pub inbound: Option<Inbound<'static>>,
    pub server_address: String,
    #[props(default)]
    pub on_close: Option<EventHandler<()>>,
    #[props(default)]
    pub on_update: Option<EventHandler<ClientAction>>,
}

#[component]
pub fn ClientDetailsModal(props: ClientDetailsModalProps) -> Element {
    let mut open = props.open;

    rsx! {
        Modal {
            open: open,
            title: "Client Details".to_string(),
            on_close: move |_| {
                open.set(false);
                if let Some(ref handler) = props.on_close {
                    handler.call(());
                }
            },
            width: "max-w-4xl".to_string(),

            if let (Some(client), Some(inbound)) = (&props.client, &props.inbound) {
                div { class: "p-6 space-y-6",
                    // Header
                    div { class: "flex items-start justify-between",
                        div {
                            h2 { class: "text-xl font-bold text-white", "Client Details" }
                            p { class: "text-gray-400 text-sm", "{client.email.as_deref().unwrap_or(\"Unknown Client\")}" }
                        }
                        button {
                            class: "p-2 hover:bg-white/10 rounded-full transition-colors text-gray-400 hover:text-white",
                            onclick: move |_| open.set(false),
                            span { class: "material-symbols-outlined", "close" }
                        }
                    }

                    div { class: "grid grid-cols-1 lg:grid-cols-2 gap-6",
                        // Left Column: Lifecycle & Stats
                        div { class: "space-y-6",
                            ClientLifecycleCard {
                                client: client.clone(),
                                current_time_ms: chrono::Utc::now().timestamp_millis(),
                                on_action: props.on_update.clone(),
                            }
                        }

                        // Right Column: Subscription & Sharing
                        div { class: "space-y-6",
                            SubscriptionGenerator {}
                        }
                    }
                }
            } else {
                div { class: "p-6 text-center text-gray-400",
                    "No client selected"
                }
            }
        }
    }
}
