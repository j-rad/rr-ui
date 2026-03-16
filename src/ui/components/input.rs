use dioxus::prelude::*;

#[derive(Clone, PartialEq, Debug, Default)]
pub enum InputType {
    #[default]
    Text,
    Password,
    Email,
    Number,
    Search,
    Url,
}

impl std::fmt::Display for InputType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputType::Text => write!(f, "text"),
            InputType::Password => write!(f, "password"),
            InputType::Email => write!(f, "email"),
            InputType::Number => write!(f, "number"),
            InputType::Search => write!(f, "search"),
            InputType::Url => write!(f, "url"),
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct InputProps {
    #[props(default)]
    pub value: String,
    pub oninput: Option<EventHandler<String>>,
    #[props(default)]
    pub r#type: InputType,
    #[props(default)]
    pub placeholder: String,
    #[props(default)]
    pub class: String,
    #[props(default)]
    pub disabled: bool,
    #[props(default)]
    pub required: bool,
}

#[component]
pub fn Input(props: InputProps) -> Element {
    rsx! {
        input {
            class: "flex h-10 w-full rounded-md border border-border bg-bg px-3 py-2 text-sm ring-offset-bg file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 {props.class}",
            r#type: "{props.r#type}",
            value: "{props.value}",
            placeholder: "{props.placeholder}",
            disabled: props.disabled,
            required: props.required,
            oninput: move |evt| {
                if let Some(handler) = &props.oninput {
                    handler.call(evt.value());
                }
            }
        }
    }
}
