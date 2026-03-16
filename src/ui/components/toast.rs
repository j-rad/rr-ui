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
        self.add(message, ToastType::Success);
    }

    pub fn error(&mut self, message: impl Into<String>) {
        self.add(message, ToastType::Error);
    }

    pub fn warning(&mut self, message: impl Into<String>) {
        self.add(message, ToastType::Warning);
    }

    pub fn info(&mut self, message: impl Into<String>) {
        self.add(message, ToastType::Info);
    }

    fn add(&mut self, message: impl Into<String>, toast_type: ToastType) {
        let id = TOAST_ID.fetch_add(1, Ordering::SeqCst);
        let toast = Toast {
            id,
            message: message.into(),
            toast_type,
        };

        self.toasts.write().push(toast);

        // Auto-dismiss after 5 seconds
        let mut toasts = self.toasts;
        spawn(async move {
            #[cfg(feature = "web")]
            crate::ui::sleep::sleep(5000 as u64).await;
            #[cfg(not(feature = "web"))]
            tokio::time::sleep(std::time::Duration::from_millis(5000)).await;

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
                }
            }
        }
    }
}

#[component]
fn ToastItem(id: u64, message: String, toast_type: ToastType) -> Element {
    let mut store = use_context::<ToastStore>();

    let dismiss = move |_| {
        store.dismiss(id);
    };

    let class = format!("toast {}", toast_type.css_class());

    rsx! {
        div { class: "{class}",
            div { class: "toast-icon",
                match toast_type {
                    ToastType::Success => rsx! { CheckCircle { size: 20 } },
                    ToastType::Error => rsx! { XCircle { size: 20 } },
                    ToastType::Warning => rsx! { AlertTriangle { size: 20 } },
                    ToastType::Info => rsx! { Info { size: 20 } },
                }
            }
            div { class: "toast-message", "{message}" }
            button { class: "toast-close", onclick: dismiss, X { size: 16 } }
        }
    }
}
