// src/ui/pages/migration.rs
//! Migration Wizard Page
//!
//! Handles automated migration from legacy X-UI SQLite database to SurrealDB.
//! Provides a drag-and-drop upload area, phased progress bar, and streaming log output.
//! Calls the `run_migration` server function for real database-to-database import.

use crate::ui::server_fns::{MigrationResult, run_migration};
use dioxus::prelude::*;

/// Represents a single step in the migration pipeline.
#[derive(Clone, PartialEq)]
enum MigrationPhase {
    Idle,
    Uploading,
    Migrating,
    Complete,
    Failed,
}

impl MigrationPhase {
    fn label(&self) -> &'static str {
        match self {
            Self::Idle => "Waiting for file",
            Self::Uploading => "Uploading database…",
            Self::Migrating => "Running migration…",
            Self::Complete => "Migration complete",
            Self::Failed => "Migration failed",
        }
    }

    fn ordinal(&self) -> u8 {
        match self {
            Self::Idle => 0,
            Self::Uploading => 1,
            Self::Migrating => 2,
            Self::Complete => 3,
            Self::Failed => 3,
        }
    }

    fn total_steps() -> u8 {
        3
    }
}

/// Migration statistics gathered during the import.
#[derive(Clone, Default)]
struct MigrationStats {
    inbounds_found: usize,
    inbounds_migrated: usize,
    inbounds_skipped: usize,
    inbounds_failed: usize,
    traffic_found: usize,
    traffic_migrated: usize,
    traffic_skipped: usize,
    traffic_failed: usize,
    total_users: usize,
}

impl From<MigrationResult> for MigrationStats {
    fn from(r: MigrationResult) -> Self {
        Self {
            inbounds_found: r.inbounds_found,
            inbounds_migrated: r.inbounds_migrated,
            inbounds_skipped: r.inbounds_skipped,
            inbounds_failed: r.inbounds_failed,
            traffic_found: r.traffic_found,
            traffic_migrated: r.traffic_migrated,
            traffic_skipped: r.traffic_skipped,
            traffic_failed: r.traffic_failed,
            total_users: r.total_users,
        }
    }
}

#[component]
pub fn MigrationPage() -> Element {
    let mut phase = use_signal(|| MigrationPhase::Idle);
    let mut progress = use_signal(|| 0.0_f64);
    let mut logs = use_signal(Vec::<String>::new);
    let mut stats = use_signal(MigrationStats::default);
    let mut selected_file = use_signal(|| Option::<String>::None);
    let mut drag_over = use_signal(|| false);
    let mut surreal_url = use_signal(|| "ws://localhost:8000".to_string());
    let mut namespace = use_signal(|| "xui".to_string());
    let mut database = use_signal(|| "panel".to_string());

    // Migration pipeline — calls the run_migration server function
    let start_migration = move |_| {
        phase.set(MigrationPhase::Uploading);
        progress.set(0.0);
        logs.write().clear();
        stats.set(MigrationStats::default());

        let file_name = selected_file
            .read()
            .clone()
            .unwrap_or_else(|| "x-ui.db".into());
        let surreal = surreal_url.read().clone();
        let ns = namespace.read().clone();
        let db = database.read().clone();

        spawn(async move {
            // Phase 1: Upload acknowledgment
            logs.write()
                .push(format!("📁 Selected database: {file_name}"));
            progress.set(15.0);

            // Phase 2: Run the actual migration
            phase.set(MigrationPhase::Migrating);
            logs.write().push(format!(
                "🔄 Connecting to SurrealDB at {surreal} ({ns}/{db})…"
            ));
            progress.set(25.0);

            match run_migration(file_name.clone(), surreal, ns, db).await {
                Ok(result) => {
                    // Log inbound phase
                    logs.write().push(format!(
                        "📦 Inbounds: {} found, {} migrated, {} skipped, {} failed",
                        result.inbounds_found,
                        result.inbounds_migrated,
                        result.inbounds_skipped,
                        result.inbounds_failed
                    ));
                    progress.set(60.0);

                    // Log traffic phase
                    logs.write().push(format!(
                        "📦 Traffic: {} found, {} migrated, {} skipped, {} failed",
                        result.traffic_found,
                        result.traffic_migrated,
                        result.traffic_skipped,
                        result.traffic_failed
                    ));
                    logs.write()
                        .push(format!("👥 Total users discovered: {}", result.total_users));
                    progress.set(90.0);

                    // Log any errors
                    for err in &result.errors {
                        logs.write().push(format!("  ❌ {err}"));
                    }

                    stats.set(MigrationStats::from(result.clone()));

                    if result.inbounds_failed == 0 && result.traffic_failed == 0 {
                        progress.set(100.0);
                        phase.set(MigrationPhase::Complete);
                        logs.write().push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".into());
                        logs.write()
                            .push("🎉 Migration completed successfully!".into());
                    } else {
                        progress.set(100.0);
                        phase.set(MigrationPhase::Complete);
                        logs.write().push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".into());
                        logs.write()
                            .push("⚠ Migration completed with errors.".into());
                    }
                }
                Err(e) => {
                    logs.write().push(format!("❌ Migration failed: {e}"));
                    progress.set(100.0);
                    phase.set(MigrationPhase::Failed);
                }
            }
        });
    };

    let handle_file_selected = move |evt: Event<FormData>| {
        if let Some(name) = evt.data().value().split('\\').last() {
            let trimmed = name.split('/').last().unwrap_or(name);
            selected_file.set(Some(trimmed.to_string()));
        }
    };

    let current_phase = phase.read().clone();
    let current_progress = *progress.read();
    let is_idle = current_phase == MigrationPhase::Idle;
    let is_running = !is_idle
        && current_phase != MigrationPhase::Complete
        && current_phase != MigrationPhase::Failed;
    let is_complete = current_phase == MigrationPhase::Complete;
    let is_failed = current_phase == MigrationPhase::Failed;

    rsx! {
        div { class: "p-6 space-y-6 max-w-5xl mx-auto animate-fade-in",

            // Header
            div { class: "bg-gradient-to-br from-[#0d1117] via-[#161b22] to-[#0d1117] border border-white/[0.06] rounded-2xl p-8 backdrop-blur-xl shadow-2xl",
                div { class: "flex items-center gap-4 mb-3",
                    div { class: "w-12 h-12 rounded-xl bg-gradient-to-br from-blue-500/20 to-purple-500/20 border border-blue-400/20 flex items-center justify-center text-2xl",
                        "🔄"
                    }
                    div {
                        h1 { class: "text-2xl font-bold text-white tracking-tight", "3x-ui Migration Wizard" }
                        p { class: "text-sm text-gray-400 mt-0.5",
                            "Import your legacy SQLite database — non-destructive, duplicate-safe"
                        }
                    }
                }

                // Phase stepper
                div { class: "flex items-center gap-1 mt-6 mb-2",
                    for i in 0..=MigrationPhase::total_steps() {
                        div {
                            class: if current_phase.ordinal() > i {
                                "h-1 flex-1 rounded-full bg-green-400/80 transition-all duration-500"
                            } else if current_phase.ordinal() == i {
                                "h-1 flex-1 rounded-full bg-blue-400 animate-pulse transition-all duration-500"
                            } else {
                                "h-1 flex-1 rounded-full bg-white/10 transition-all duration-500"
                            },
                        }
                    }
                }
                p { class: "text-xs text-gray-500 text-right", "{current_phase.label()}" }
            }

            // Upload Area + Config (visible when idle)
            if is_idle {
                div { class: "grid grid-cols-1 lg:grid-cols-3 gap-4",
                    // File picker — takes 2 cols
                    div {
                        class: if *drag_over.read() {
                            "lg:col-span-2 border-2 border-dashed border-blue-400 rounded-2xl p-12 text-center bg-blue-500/5 backdrop-blur-xl transition-all duration-300 cursor-pointer"
                        } else {
                            "lg:col-span-2 border-2 border-dashed border-white/10 rounded-2xl p-12 text-center hover:border-blue-400/40 hover:bg-white/[0.02] backdrop-blur-xl transition-all duration-300 cursor-pointer"
                        },
                        ondragover: move |evt| {
                            evt.prevent_default();
                            drag_over.set(true);
                        },
                        ondragleave: move |_| {
                            drag_over.set(false);
                        },

                        div { class: "text-5xl mb-4 opacity-40", "🗄️" }
                        h3 { class: "text-lg font-semibold text-white mb-1",
                            if selected_file.read().is_some() {
                                "📎 {selected_file.read().as_deref().unwrap_or(\"\")}"
                            } else {
                                "Select or drop your x-ui.db file"
                            }
                        }
                        p { class: "text-sm text-gray-500 mb-4", "Supports .db and .sqlite files from 3x-ui / x-ui" }

                        label { class: "inline-block px-6 py-2.5 bg-white/5 border border-white/10 rounded-lg text-sm text-gray-300 hover:bg-white/10 hover:border-white/20 transition cursor-pointer",
                            "Browse Files"
                            input {
                                r#type: "file",
                                class: "hidden",
                                accept: ".db,.sqlite",
                                onchange: handle_file_selected,
                            }
                        }
                    }

                    // SurrealDB config panel
                    div { class: "bg-[#0d1117] border border-white/[0.06] rounded-2xl p-6 space-y-4",
                        h3 { class: "text-sm font-bold text-white uppercase tracking-wider mb-2", "Target SurrealDB" }

                        div { class: "space-y-3",
                            div {
                                label { class: "block text-xs text-gray-500 mb-1", "WebSocket URL" }
                                input {
                                    class: "w-full bg-black/40 border border-white/10 rounded-lg px-3 py-2 text-sm text-gray-200 font-mono focus:border-blue-400/50 focus:outline-none transition",
                                    value: "{surreal_url}",
                                    oninput: move |evt| surreal_url.set(evt.value()),
                                }
                            }
                            div {
                                label { class: "block text-xs text-gray-500 mb-1", "Namespace" }
                                input {
                                    class: "w-full bg-black/40 border border-white/10 rounded-lg px-3 py-2 text-sm text-gray-200 font-mono focus:border-blue-400/50 focus:outline-none transition",
                                    value: "{namespace}",
                                    oninput: move |evt| namespace.set(evt.value()),
                                }
                            }
                            div {
                                label { class: "block text-xs text-gray-500 mb-1", "Database" }
                                input {
                                    class: "w-full bg-black/40 border border-white/10 rounded-lg px-3 py-2 text-sm text-gray-200 font-mono focus:border-blue-400/50 focus:outline-none transition",
                                    value: "{database}",
                                    oninput: move |evt| database.set(evt.value()),
                                }
                            }
                        }
                    }
                }
            }

            // Progress + Logs Panel
            if !is_idle {
                div { class: "bg-[#0d1117] border border-white/[0.06] rounded-2xl overflow-hidden shadow-xl",

                    // Progress header
                    div { class: "px-6 py-4 border-b border-white/[0.06] flex items-center justify-between",
                        div { class: "flex items-center gap-3",
                            if is_running {
                                div { class: "w-2 h-2 rounded-full bg-blue-400 animate-pulse" }
                            } else if is_complete {
                                div { class: "w-2 h-2 rounded-full bg-green-400" }
                            } else if is_failed {
                                div { class: "w-2 h-2 rounded-full bg-red-400" }
                            }
                            span { class: "text-sm font-medium text-white", "{current_phase.label()}" }
                        }
                        span { class: "text-xs font-mono text-gray-400", "{current_progress:.0}%" }
                    }

                    // Progress bar
                    div { class: "mx-6 mt-3",
                        div { class: "w-full bg-white/5 rounded-full h-1.5 overflow-hidden",
                            div {
                                class: if is_complete {
                                    "h-1.5 rounded-full bg-gradient-to-r from-green-400 to-emerald-500 transition-all duration-500 ease-out"
                                } else if is_failed {
                                    "h-1.5 rounded-full bg-gradient-to-r from-red-500 to-rose-500 transition-all duration-500 ease-out"
                                } else {
                                    "h-1.5 rounded-full bg-gradient-to-r from-blue-500 to-purple-500 transition-all duration-300 ease-out"
                                },
                                style: "width: {current_progress}%",
                            }
                        }
                    }

                    // Stats row (visible after some progress)
                    if current_progress > 30.0 {
                        div { class: "grid grid-cols-2 md:grid-cols-5 gap-4 px-6 py-4",
                            div { class: "text-center",
                                div { class: "text-lg font-bold text-white", "{stats.read().inbounds_migrated}" }
                                div { class: "text-[10px] text-gray-500 uppercase tracking-wider", "Inbounds" }
                            }
                            div { class: "text-center",
                                div { class: "text-lg font-bold text-blue-400", "{stats.read().total_users}" }
                                div { class: "text-[10px] text-gray-500 uppercase tracking-wider", "Users" }
                            }
                            div { class: "text-center",
                                div { class: "text-lg font-bold text-purple-400", "{stats.read().traffic_migrated}" }
                                div { class: "text-[10px] text-gray-500 uppercase tracking-wider", "Traffic Records" }
                            }
                            div { class: "text-center",
                                div { class: "text-lg font-bold text-yellow-400", "{stats.read().inbounds_skipped + stats.read().traffic_skipped}" }
                                div { class: "text-[10px] text-gray-500 uppercase tracking-wider", "Duplicates Skipped" }
                            }
                            div { class: "text-center",
                                div { class: "text-lg font-bold text-red-400", "{stats.read().inbounds_failed + stats.read().traffic_failed}" }
                                div { class: "text-[10px] text-gray-500 uppercase tracking-wider", "Failures" }
                            }
                        }
                    }

                    // Log output
                    div { class: "mx-6 mb-6 bg-black/40 rounded-xl border border-white/5 p-4 h-64 overflow-y-auto font-mono text-xs leading-relaxed",
                        for (i, log) in logs.read().iter().enumerate() {
                            p {
                                key: "{i}",
                                class: if log.contains('❌') {
                                    "text-red-400"
                                } else if log.contains('⏭') || log.contains('⚠') {
                                    "text-yellow-400"
                                } else if log.contains('✓') {
                                    "text-green-400"
                                } else if log.contains('🎉') {
                                    "text-emerald-300 font-bold"
                                } else {
                                    "text-gray-400"
                                },
                                "{log}"
                            }
                        }
                    }
                }
            }

            // Action bar
            div { class: "flex justify-end gap-3",
                if is_idle && selected_file.read().is_some() {
                    button {
                        class: "px-8 py-3 bg-gradient-to-r from-blue-600 to-blue-500 text-white rounded-xl hover:from-blue-500 hover:to-blue-400 transition-all duration-200 font-semibold shadow-lg shadow-blue-500/20 active:scale-[0.98]",
                        onclick: start_migration,
                        "🚀 Start Migration"
                    }
                }
                if is_complete {
                    a {
                        class: "px-8 py-3 bg-gradient-to-r from-green-600 to-emerald-500 text-white rounded-xl hover:from-green-500 hover:to-emerald-400 transition-all duration-200 font-semibold shadow-lg shadow-green-500/20 cursor-pointer active:scale-[0.98]",
                        href: "/panel",
                        "✨ Go to Dashboard"
                    }
                }
                if is_failed {
                    button {
                        class: "px-8 py-3 bg-gradient-to-r from-red-600 to-rose-500 text-white rounded-xl hover:from-red-500 hover:to-rose-400 transition-all duration-200 font-semibold shadow-lg shadow-red-500/20 active:scale-[0.98]",
                        onclick: move |_| {
                            phase.set(MigrationPhase::Idle);
                            progress.set(0.0);
                        },
                        "🔄 Retry"
                    }
                }
                if is_running {
                    button {
                        class: "px-8 py-3 bg-white/5 border border-white/10 text-gray-400 rounded-xl cursor-not-allowed font-medium",
                        disabled: true,
                        "⏳ Migrating…"
                    }
                }
            }
        }
    }
}
