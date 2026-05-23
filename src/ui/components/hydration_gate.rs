//! Hydration Gate Component
//!
//! Provides a brief loading screen during the very first initialization
//! (SyncStatus::Initial / Syncing), then renders the UI immediately.
//!
//! **Design rationale:** blocking the entire panel on gRPC connectivity
//! makes the admin UI inaccessible when the core binary is absent. Instead,
//! we let the UI render in degraded mode and surface connectivity state via
//! the sidebar status indicator.
//!
//! The gate uses a **2-second hard timeout**: even if the core never responds,
//! the gate opens and the admin panel renders in degraded mode. This ensures
//! the panel is always reachable for configuration and diagnostics.

use dioxus::prelude::*;
use crate::ui::state::UIState;
use crate::ui::app::SyncStatus;

/// Maximum milliseconds to show the loading spinner before force-opening the gate.
const GATE_TIMEOUT_MS: u64 = 2000;

#[derive(Props, Clone, PartialEq)]
pub struct HydrationGateProps {
    pub children: Element,
}

/// Renders children once the sync loop has left the `Initial` state **or** a
/// hard timeout has elapsed — whichever comes first.
///
/// This guarantees the admin panel is accessible even when the RustRay core
/// binary is absent, offline, or slow to respond.
#[component]
pub fn HydrationGate(props: HydrationGateProps) -> Element {
    let state = use_context::<UIState>();
    let status = state.status;
    let mut timeout_fired = use_signal(|| false);

    // Start a one-shot timer that force-opens the gate after GATE_TIMEOUT_MS.
    use_hook(move || {
        spawn(async move {
            crate::ui::sleep::sleep(GATE_TIMEOUT_MS).await;
            timeout_fired.set(true);
        });
    });

    // Show the spinner only while we are in the very first boot phase
    // AND the hard timeout has not yet fired.
    let is_booting = matches!(*status.read(), SyncStatus::Initial | SyncStatus::Syncing);

    if is_booting && !*timeout_fired.read() {
        return rsx! {
            // StealthLoader: Absolute center, zero layout impact, no JS.
            div { class: "fixed inset-0 z-[100] flex items-center justify-center bg-[#030712]",
                style {
                    ".stealth-loader {{
                        width: 40px;
                        height: 40px;
                        border: 2px solid rgba(6, 182, 212, 0.1);
                        border-top: 2px solid #06b6d4;
                        border-radius: 50%;
                        animation: spin 0.8s linear infinite;
                    }}
                    @keyframes spin {{
                        0% {{ transform: rotate(0deg); }}
                        100% {{ transform: rotate(360deg); }}
                    }}"
                }
                div { class: "stealth-loader" }
            }
        };
    }

    props.children
}
