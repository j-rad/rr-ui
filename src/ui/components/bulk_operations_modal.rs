// src/ui/components/bulk_operations_modal.rs
//! Bulk Operations Modal Component
//!
//! Provides UI for mass client management operations

use crate::domain::bulk_operations::*;
use dioxus::prelude::*;

/// Props for bulk operations modal
#[derive(Props, Clone, PartialEq)]
pub struct BulkOperationsModalProps {
    /// IDs of selected clients
    pub selected_ids: Vec<String>,
    /// Callback when modal is closed
    pub on_close: EventHandler<()>,
    /// Callback when operation completes
    pub on_complete: EventHandler<BulkOperationProgress>,
}

/// Bulk Operations Modal Component
#[component]
pub fn BulkOperationsModal(props: BulkOperationsModalProps) -> Element {
    let mut selected_operation = use_signal(|| "enable".to_string());
    let mut days_input = use_signal(|| 30i64);
    let mut quota_input = use_signal(|| 100u64);
    let mut is_processing = use_signal(|| false);
    let mut progress = use_signal(|| None::<BulkOperationProgress>);
    let mut confirmation_required = use_signal(|| false);

    let client_count = props.selected_ids.len();

    let get_operation = move || -> BulkOperation {
        match selected_operation().as_str() {
            "enable" => BulkOperation::Enable,
            "disable" => BulkOperation::Disable,
            "extend_expiry" => BulkOperation::ExtendExpiry { days: days_input() },
            "set_quota" => BulkOperation::SetQuota {
                total_gb: quota_input(),
            },
            "add_quota" => BulkOperation::AddQuota {
                additional_gb: quota_input(),
            },
            "reset_traffic" => BulkOperation::ResetTraffic,
            "delete" => BulkOperation::Delete,
            _ => BulkOperation::Enable,
        }
    };

    let needs_confirmation = move || {
        let op = get_operation();
        op.is_destructive()
    };

    let execute_operation = {
        let selected_ids = props.selected_ids.clone();
        let on_complete = props.on_complete.clone();

        move |_| {
            if needs_confirmation() && !confirmation_required() {
                confirmation_required.set(true);
                return;
            }

            is_processing.set(true);

            let ids = selected_ids.clone();
            let operation = get_operation();
            let on_complete = on_complete.clone();

            spawn(async move {
                // Simulate processing for now (server function would go here)
                let mut prog = BulkOperationProgress::new(ids.len());

                for (i, _id) in ids.iter().enumerate() {
                    // Simulate batch delay
                    if i > 0 && i % 50 == 0 {
                        #[cfg(feature = "web")]
                        {
                            gloo_timers::future::TimeoutFuture::new(100).await;
                        }
                    }
                    prog.record_success();
                    progress.set(Some(prog.clone()));
                }

                prog.mark_complete();
                progress.set(Some(prog.clone()));
                is_processing.set(false);
                on_complete.call(prog);
            });
        }
    };

    rsx! {
        div { class: "modal-overlay",
            onclick: move |e| {
                // Only close if clicking overlay, not modal content
                e.stop_propagation();
            },

            div { class: "modal card bulk-operations-modal",
                onclick: move |e| e.stop_propagation(),

                // Header
                div { class: "modal-header",
                    h2 { class: "modal-title", "Bulk Operations" }
                    button {
                        class: "modal-close",
                        onclick: move |_| props.on_close.call(()),
                        "×"
                    }
                }

                // Content
                div { class: "modal-content",
                    // Selection info
                    div { class: "bulk-info",
                        i { class: "fas fa-users" }
                        span { "{client_count} clients selected" }
                    }

                    // Operation selector
                    div { class: "form-group",
                        label { "Operation" }
                        select {
                            class: "form-select",
                            value: "{selected_operation}",
                            onchange: move |e| {
                                selected_operation.set(e.value());
                                confirmation_required.set(false);
                            },

                            optgroup { label: "Status",
                                option { value: "enable", "Enable All" }
                                option { value: "disable", "Disable All" }
                            }
                            optgroup { label: "Expiry",
                                option { value: "extend_expiry", "Extend Expiry" }
                            }
                            optgroup { label: "Traffic",
                                option { value: "set_quota", "Set Quota" }
                                option { value: "add_quota", "Add Quota" }
                                option { value: "reset_traffic", "Reset Traffic ⚠️" }
                            }
                            optgroup { label: "Danger Zone",
                                option { value: "delete", "Delete All ⚠️" }
                            }
                        }
                    }

                    // Conditional inputs based on operation
                    if selected_operation() == "extend_expiry" {
                        div { class: "form-group",
                            label { "Days to Add" }
                            input {
                                r#type: "number",
                                class: "form-input",
                                value: "{days_input}",
                                min: "1",
                                max: "365",
                                onchange: move |e| {
                                    if let Ok(v) = e.value().parse::<i64>() {
                                        days_input.set(v);
                                    }
                                }
                            }
                        }
                    }

                    if selected_operation() == "set_quota" || selected_operation() == "add_quota" {
                        div { class: "form-group",
                            label { "Traffic (GB)" }
                            input {
                                r#type: "number",
                                class: "form-input",
                                value: "{quota_input}",
                                min: "1",
                                onchange: move |e| {
                                    if let Ok(v) = e.value().parse::<u64>() {
                                        quota_input.set(v);
                                    }
                                }
                            }
                        }
                    }

                    // Confirmation warning for destructive operations
                    if confirmation_required() {
                        div { class: "alert alert-danger",
                            i { class: "fas fa-exclamation-triangle" }
                            span { "This action cannot be undone. Click Execute again to confirm." }
                        }
                    }

                    // Progress bar
                    if let Some(prog) = progress() {
                        div { class: "progress-section",
                            div { class: "progress-bar",
                                div {
                                    class: "progress-fill",
                                    style: "width: {prog.percent_complete()}%"
                                }
                            }
                            div { class: "progress-stats",
                                span { "{prog.processed}/{prog.total}" }
                                span { class: "text-success", "✓ {prog.succeeded}" }
                                if prog.failed > 0 {
                                    span { class: "text-danger", "✗ {prog.failed}" }
                                }
                            }
                        }
                    }
                }

                // Footer
                div { class: "modal-footer",
                    button {
                        class: "btn btn-secondary",
                        onclick: move |_| props.on_close.call(()),
                        disabled: is_processing(),
                        "Cancel"
                    }
                    button {
                        class: if needs_confirmation() { "btn btn-danger" } else { "btn btn-primary" },
                        onclick: execute_operation,
                        disabled: is_processing() || client_count == 0,

                        if is_processing() {
                            i { class: "fas fa-spinner fa-spin" }
                            " Processing..."
                        } else if confirmation_required() {
                            "Confirm Delete"
                        } else {
                            "Execute"
                        }
                    }
                }
            }
        }
    }
}
