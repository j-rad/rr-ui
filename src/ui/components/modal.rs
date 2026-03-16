//! Modal Component
//!
//! Modal dialog matching the Svelte Modal.svelte component.

use dioxus::prelude::*;

use super::icons::X;

#[derive(Props, Clone, PartialEq)]
pub struct ModalProps {
    /// Whether the modal is open
    pub open: Signal<bool>,
    /// Modal title
    pub title: String,
    /// Modal width (CSS value)
    #[props(default = "520px".to_string())]
    pub width: String,
    /// Called when modal is closed
    #[props(default)]
    pub on_close: Option<EventHandler<()>>,
    /// Main modal content
    pub children: Element,
    /// Footer content (buttons, etc.)
    #[props(default)]
    pub footer: Option<Element>,
}

#[component]
pub fn Modal(props: ModalProps) -> Element {
    let mut open = props.open;
    let on_close = props.on_close.clone();

    let handle_close = {
        let on_close = on_close.clone();
        move |_: MouseEvent| {
            open.set(false);
            if let Some(ref handler) = on_close {
                handler.call(());
            }
        }
    };

    let handle_overlay_click = {
        let on_close = on_close.clone();
        move |e: MouseEvent| {
            // Close when clicking overlay background
            // Note: In Dioxus, checking target == currentTarget isn't straightforward
            // The modal content has separate onclick that doesn't bubble
            open.set(false);
            if let Some(ref handler) = on_close {
                handler.call(());
            }
        }
    };

    // Prevent click from propagating through modal content
    let stop_propagation = move |e: MouseEvent| {
        e.stop_propagation();
    };

    if !open() {
        return rsx! {};
    }

    rsx! {
        div {
            class: "modal-overlay",
            onclick: handle_overlay_click,
            div {
                class: "modal",
                style: "max-width: {props.width};",
                onclick: stop_propagation,
                // Header
                div { class: "modal-header",
                    span { class: "modal-title", "{props.title}" }
                    button { class: "btn btn-icon", onclick: handle_close, X {} }
                }

                // Body
                div { class: "modal-body", {props.children} }

                // Footer
                if let Some(footer) = props.footer {
                    div { class: "modal-footer", {footer} }
                }
            }
        }
    }
}
