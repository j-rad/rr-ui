//! Toast Notification Component
//!
//! Toast notifications matching the Svelte Toast.svelte component.

use dioxus::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};

use super::icons::{AlertTriangle, CheckCircle, Info, X, XCircle};

static TOAST_ID: AtomicU64 = AtomicU64::new(0);

/// Toast notification type
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ToastType {
    Success,
    Error,
    Warning,
    Info,
}

impl ToastType {
    fn css_class(&self) -> &'static str {
        match self {
            ToastType::Success => "toast-success",
            ToastType::Error => "toast-error",
            ToastType::Warning => "toast-warning",
            ToastType::Info => "toast-info",
        }
    }
}

/// Individual toast notification
#[derive(Clone, PartialEq)]
pub struct Toast {
    pub id: u64,
    pub message: String,
    pub toast_type: ToastType,
    pub tactical: Option<crate::ui::error_bridge::TacticalMessage>,
}

/// Toast store for managing notifications
#[derive(Clone)]
pub struct ToastStore {
    pub toasts: Signal<Vec<Toast>>,
}

impl ToastStore {
    pub fn new() -> Self {
        Self {
            toasts: Signal::new(Vec::new()),
        }
    }

    pub fn success(&mut self, message: impl Into<String>) {
        self.add(message, ToastType::Success, None);
    }

    pub fn error(&mut self, message: impl Into<String>) {
        self.add(message, ToastType::Error, None);
    }

    pub fn warning(&mut self, message: impl Into<String>) {
        self.add(message, ToastType::Warning, None);
    }

    pub fn info(&mut self, message: impl Into<String>) {
        self.add(message, ToastType::Info, None);
    }

    pub fn tactical(&mut self, err: crate::ui::error_bridge::TacticalMessage) {
        let message = format!("{}: {}", err.title, err.message);
        let toast_type = match err.severity {
            crate::ui::error_bridge::Level::Critical | crate::ui::error_bridge::Level::Error => ToastType::Error,
            crate::ui::error_bridge::Level::Warning => ToastType::Warning,
            crate::ui::error_bridge::Level::Info => ToastType::Info,
        };
        self.add(message, toast_type, Some(err));
    }

    fn add(&mut self, message: impl Into<String>, toast_type: ToastType, tactical: Option<crate::ui::error_bridge::TacticalMessage>) {
        let id = TOAST_ID.fetch_add(1, Ordering::SeqCst);
        let toast = Toast {
            id,
            message: message.into(),
            toast_type,
            tactical,
        };

        self.toasts.write().push(toast);

        // Auto-dismiss after 5 seconds
        let mut toasts = self.toasts;
        spawn(async move {
            crate::ui::sleep::sleep(5000 as u64).await;

            toasts.write().retain(|t| t.id != id);
        });
    }

    pub fn dismiss(&mut self, id: u64) {
        self.toasts.write().retain(|t| t.id != id);
    }
}

impl Default for ToastStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Toast container component - render once at app root
#[component]
pub fn ToastContainer() -> Element {
    let store = use_context::<ToastStore>();
    let toasts = store.toasts;

    rsx! {
        div { class: "toast-container",
            for toast in toasts.read().iter() {
                ToastItem {
                    key: "{toast.id}",
                    id: toast.id,
                    message: toast.message.clone(),
                    toast_type: toast.toast_type,
                    tactical: toast.tactical.clone(),
                }
            }
        }
    }
}

#[component]
fn ToastItem(id: u64, message: String, toast_type: ToastType, tactical: Option<crate::ui::error_bridge::TacticalMessage>) -> Element {
    let mut store = use_context::<ToastStore>();

    let dismiss = move |_| {
        store.dismiss(id);
    };

    let class = format!("toast {}", toast_type.css_class());

    rsx! {
        div { class: "{class} flex-col !items-start gap-1 p-4 min-w-[320px] max-w-md",
            div { class: "flex items-center gap-3 w-full",
                div { class: "toast-icon shrink-0",
                    match toast_type {
                        ToastType::Success => rsx! { CheckCircle { size: 20 } },
                        ToastType::Error => rsx! { XCircle { size: 20 } },
                        ToastType::Warning => rsx! { AlertTriangle { size: 20 } },
                        ToastType::Info => rsx! { Info { size: 20 } },
                    }
                }
                div { class: "toast-message font-semibold flex-1", "{message}" }
                button { class: "toast-close shrink-0", onclick: dismiss, X { size: 16 } }
            }
            
            if let Some(err) = tactical {
                div { class: "pl-8 flex flex-col gap-2 mt-1",
                    p { class: "text-xs opacity-80 leading-relaxed", "{err.message}" }
                    div { class: "bg-black/20 p-2 rounded border border-white/5",
                        p { class: "text-[10px] uppercase tracking-wider font-bold text-cyan-400 mb-0.5", "Tactical Suggestion" }
                        p { class: "text-xs font-mono", "{err.suggestion}" }
                    }
                }
            }
        }
    }
}
