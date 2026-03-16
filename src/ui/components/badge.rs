use dioxus::prelude::*;

#[derive(Clone, PartialEq, Debug, Default)]
pub enum BadgeVariant {
    #[default]
    Default,
    Secondary,
    Destructive,
    Outline,
    Success,
    Warning,
    Neutral,
}

#[derive(Props, Clone, PartialEq)]
pub struct BadgeProps {
    #[props(default)]
    pub variant: BadgeVariant,
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn Badge(props: BadgeProps) -> Element {
    let base_classes = "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2";

    let variant_classes = match props.variant {
        BadgeVariant::Default => {
            "border-transparent bg-primary text-primary-foreground hover:bg-primary/80"
        }
        BadgeVariant::Secondary => {
            "border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80"
        }
        BadgeVariant::Destructive => {
            "border-transparent bg-destructive text-destructive-foreground hover:bg-destructive/80"
        }
        BadgeVariant::Outline => "text-foreground",
        BadgeVariant::Success => "border-transparent bg-green-500 text-white hover:bg-green-600",
        BadgeVariant::Warning => "border-transparent bg-yellow-500 text-white hover:bg-yellow-600",
        BadgeVariant::Neutral => "border-transparent bg-gray-500 text-white hover:bg-gray-600",
    };
    rsx! {
        div {
            class: "{base_classes} {variant_classes} {props.class}",
            {props.children}
        }
    }
}
