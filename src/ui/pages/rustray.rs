//! RustRay Config Page

use crate::ui::components::card::Card;
use dioxus::prelude::*;

#[component]
pub fn RustRayPage() -> Element {
    let template = r#"{
  "log": { ... },
  "inbounds": [ ... ],
  "outbounds": [ ... ]
}"#;

    rsx! {
        h1 { class: "text-xl font-bold mb-16", "RustRay Configuration" }

        Card { title: "RustRay Template".to_string(),
            pre { class: "p-16", style: "background: var(--color-bg-tertiary); border-radius: 4px; overflow: auto;",
                code { "{template}" }
            }
        }
    }
}
