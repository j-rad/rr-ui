//! Backups Page

use crate::ui::components::card::Card;
use dioxus::prelude::*;

#[component]
pub fn BackupsPage() -> Element {
    rsx! {
        div { class: "flex justify-between items-center mb-16",
            h1 { class: "text-xl font-bold", "Backups" }
            button { class: "btn btn-primary", "Create Backup" }
        }

        Card {
            div { class: "table-container",
                table { class: "table",
                    thead {
                        tr {
                            th { "Name" }
                            th { "Date" }
                            th { "Size" }
                            th { "Actions" }
                        }
                    }
                    tbody {
                        tr {
                            td { colspan: "4", class: "text-center text-secondary",
                                "No backups available"
                            }
                        }
                    }
                }
            }
        }
    }
}
