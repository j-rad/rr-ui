//! Card Component
//!
//! Reusable card container matching the Svelte Card.svelte component.

use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct CardProps {
    /// Card title
    #[props(default)]
    pub title: Option<String>,
    /// Whether the card has hover effects
    #[props(default = false)]
    pub hoverable: bool,
    /// Additional CSS classes
    #[props(default)]
    pub class: Option<String>,
    /// Main card content
    pub children: Element,
    /// Extra content in the title bar (right side)
    #[props(default)]
    pub extra: Option<Element>,
    /// Actions row at the bottom of the card
    #[props(default)]
    pub actions: Option<Element>,
}

#[component]
pub fn Card(props: CardProps) -> Element {
    let mut classes = vec![
        "relative",
        "bg-glass-bg/60",
        "backdrop-blur-xl",
        "border",
        "border-glass-border",
        "rounded-xl",
        "shadow-lg",
        "transition-all",
        "duration-300",
        "overflow-hidden",
    ];

    if props.hoverable {
        classes.push("hover:border-primary/30");
        classes.push("hover:-translate-y-1");
        classes.push("hover:shadow-glow");
    }

    if let Some(ref custom_class) = props.class {
        classes.push(custom_class);
    }

    let class_str = classes.join(" ");

    rsx! {
        div { class: "{class_str}",
            // Subtle gradient overlay
            div { class: "absolute inset-0 bg-gradient-to-br from-white/[0.03] via-transparent to-black/[0.02] pointer-events-none rounded-xl" }

            // Title bar
            if props.title.is_some() || props.extra.is_some() {
                div { class: "relative px-5 py-4 border-b border-border-light/50 flex justify-between items-center",
                    if let Some(ref title) = props.title {
                        span { class: "font-semibold text-text-main tracking-tight", "{title}" }
                    }
                    if let Some(extra) = props.extra {
                        div { class: "text-sm text-text-secondary", {extra} }
                    }
                }
            }

            // Main content
            div { class: "relative p-5", {props.children} }

            // Actions footer
            if let Some(actions) = props.actions {
                div { class: "relative px-5 py-3 border-t border-border-light/50 bg-black/10 flex justify-end gap-3", {actions} }
            }
        }
    }
}

/// Card action button for the actions row
#[derive(Props, Clone, PartialEq)]
pub struct CardActionProps {
    pub children: Element,
    #[props(default)]
    pub onclick: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub href: Option<String>,
}

#[component]
pub fn CardAction(props: CardActionProps) -> Element {
    if let Some(href) = props.href {
        rsx! {
            a { class: "card-action", href: "{href}", {props.children} }
        }
    } else {
        rsx! {
            div {
                class: "card-action",
                onclick: move |e| {
                    if let Some(handler) = &props.onclick {
                        handler.call(e);
                    }
                },
                {props.children}
            }
        }
    }
}

#[component]
pub fn CardHeader(children: Element, #[props(default)] class: String) -> Element {
    rsx! {
        div { class: "flex flex-col space-y-1.5 p-6 {class}", {children} }
    }
}

#[component]
pub fn CardTitle(children: Element, #[props(default)] class: String) -> Element {
    rsx! {
        h3 { class: "font-semibold leading-none tracking-tight {class}", {children} }
    }
}

#[component]
pub fn CardContent(children: Element, #[props(default)] class: String) -> Element {
    rsx! {
        div { class: "p-6 pt-0 {class}", {children} }
    }
}
