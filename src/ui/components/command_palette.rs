// src/ui/components/command_palette.rs
//! Command Palette component
//!
//! Global Ctrl+K / Cmd+K fuzzy search overlay for navigating users and inbounds.
//! Mounted once inside PanelLayout, always listening for the hotkey.

use dioxus::prelude::*;

/// A single search result entry.
#[derive(Clone, PartialEq)]
struct SearchEntry {
    label: String,
    sublabel: String,
    category: SearchCategory,
    route: String,
}

#[derive(Clone, PartialEq)]
enum SearchCategory {
    User,
    Inbound,
}

impl SearchCategory {
    fn badge_class(&self) -> &'static str {
        match self {
            Self::User => "bg-purple-500/20 text-purple-300 border-purple-500/30",
            Self::Inbound => "bg-blue-500/20 text-blue-300 border-blue-500/30",
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::User => "User",
            Self::Inbound => "Inbound",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            Self::User => "👤",
            Self::Inbound => "📡",
        }
    }
}

/// Performs case-insensitive fuzzy matching with contiguous substring.
fn fuzzy_match(query: &str, haystack: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let q = query.to_lowercase();
    let h = haystack.to_lowercase();
    h.contains(&q)
}

#[component]
pub fn CommandPalette() -> Element {
    let mut open = use_signal(|| false);
    let mut query = use_signal(String::new);
    let mut entries = use_signal(Vec::<SearchEntry>::new);
    let mut selected_index = use_signal(|| 0_usize);
    let navigator = use_navigator();

    // Fetch search data when palette opens
    let mut fetch_entries = move || {
        entries.set(Vec::new());
        spawn(async move {
            if let Ok(inbounds) = crate::ui::server_fns::list_inbounds().await {
                let mut results = Vec::new();

                for inbound in &inbounds {
                    // Add inbound entry
                    results.push(SearchEntry {
                        label: inbound.remark.to_string(),
                        sublabel: format!(
                            ":{} · {} · {}",
                            inbound.port,
                            inbound.protocol.as_str(),
                            inbound.tag
                        ),
                        category: SearchCategory::Inbound,
                        route: "/panel/inbounds".to_string(),
                    });

                    // Add user entries from client lists
                    if let Some(clients) = inbound.settings.clients() {
                        for client in clients {
                            let email = client.email.as_deref().unwrap_or("—").to_string();
                            let uuid = client.id.as_deref().unwrap_or("—");
                            results.push(SearchEntry {
                                label: email,
                                sublabel: format!(
                                    "{} · {}",
                                    uuid.get(..8).unwrap_or(uuid),
                                    inbound.remark
                                ),
                                category: SearchCategory::User,
                                route: "/panel/inbounds".to_string(),
                            });
                        }
                    }
                }

                entries.set(results);
            }
        });
    };

    // Keyboard listener for Ctrl+K and navigation
    let on_keydown = move |evt: Event<KeyboardData>| {
        let key = evt.data().key();
        let modifiers = evt.data().modifiers();
        let ctrl_or_meta =
            modifiers.contains(Modifiers::CONTROL) || modifiers.contains(Modifiers::META);

        if ctrl_or_meta && key == Key::Character("k".to_string()) {
            evt.prevent_default();
            let is_open = *open.read();
            if !is_open {
                fetch_entries();
            }
            open.set(!is_open);
            query.set(String::new());
            selected_index.set(0);
            return;
        }

        if *open.read() {
            match key {
                Key::Escape => {
                    open.set(false);
                }
                Key::ArrowDown => {
                    evt.prevent_default();
                    let q = query.read().clone();
                    let filtered_count = entries
                        .read()
                        .iter()
                        .filter(|e| fuzzy_match(&q, &e.label) || fuzzy_match(&q, &e.sublabel))
                        .count();
                    if filtered_count > 0 {
                        let idx = *selected_index.read();
                        selected_index.set((idx + 1) % filtered_count);
                    }
                }
                Key::ArrowUp => {
                    evt.prevent_default();
                    let q = query.read().clone();
                    let filtered_count = entries
                        .read()
                        .iter()
                        .filter(|e| fuzzy_match(&q, &e.label) || fuzzy_match(&q, &e.sublabel))
                        .count();
                    if filtered_count > 0 {
                        let idx = *selected_index.read();
                        selected_index.set(if idx == 0 {
                            filtered_count - 1
                        } else {
                            idx - 1
                        });
                    }
                }
                Key::Enter => {
                    let q = query.read().clone();
                    let filtered: Vec<_> = entries
                        .read()
                        .iter()
                        .filter(|e| fuzzy_match(&q, &e.label) || fuzzy_match(&q, &e.sublabel))
                        .cloned()
                        .collect();
                    let idx = *selected_index.read();
                    if let Some(entry) = filtered.get(idx) {
                        navigator.push(entry.route.clone());
                        open.set(false);
                    }
                }
                _ => {}
            }
        }
    };

    if !*open.read() {
        // Invisible keyboard catcher
        return rsx! {
            div {
                tabindex: "0",
                class: "fixed w-0 h-0 overflow-hidden opacity-0",
                onkeydown: on_keydown,
                autofocus: true,
            }
        };
    }

    let current_query = query.read().clone();
    let filtered: Vec<(usize, SearchEntry)> = entries
        .read()
        .iter()
        .filter(|e| {
            fuzzy_match(&current_query, &e.label) || fuzzy_match(&current_query, &e.sublabel)
        })
        .take(20)
        .cloned()
        .enumerate()
        .collect();
    let sel = *selected_index.read();

    rsx! {
        // Backdrop
        div {
            class: "fixed inset-0 z-50 bg-black/60 backdrop-blur-sm flex items-start justify-center pt-[15vh] animate-fade-in",
            onclick: move |_| open.set(false),
            onkeydown: on_keydown,

            // Palette container
            div {
                class: "w-full max-w-xl bg-[#0d1117] border border-white/[0.08] rounded-2xl shadow-2xl overflow-hidden animate-slide-up",
                onclick: move |evt| evt.stop_propagation(),

                // Search input
                div { class: "flex items-center gap-3 px-5 py-4 border-b border-white/[0.06]",
                    span { class: "text-gray-500 text-lg", "🔍" }
                    input {
                        class: "flex-1 bg-transparent text-white text-sm placeholder-gray-500 focus:outline-none",
                        placeholder: "Search users, inbounds…",
                        autofocus: true,
                        value: "{query}",
                        oninput: move |evt| {
                            query.set(evt.value());
                            selected_index.set(0);
                        },
                    }
                    span { class: "text-[10px] text-gray-600 bg-white/5 px-2 py-0.5 rounded border border-white/10 font-mono", "ESC" }
                }

                // Results
                div { class: "max-h-80 overflow-y-auto",
                    if filtered.is_empty() {
                        div { class: "px-5 py-10 text-center text-gray-500 text-sm",
                            if current_query.is_empty() {
                                "Loading…"
                            } else {
                                "No results for \"{current_query}\""
                            }
                        }
                    }

                    for (idx, entry) in filtered.iter() {
                        div {
                            key: "{idx}",
                            class: if *idx == sel {
                                "px-5 py-3 flex items-center gap-3 bg-blue-500/10 border-l-2 border-blue-400 cursor-pointer transition-colors duration-150"
                            } else {
                                "px-5 py-3 flex items-center gap-3 hover:bg-white/[0.03] border-l-2 border-transparent cursor-pointer transition-colors duration-150"
                            },
                            onclick: {
                                let route = entry.route.clone();
                                move |_| {
                                    navigator.push(route.clone());
                                    open.set(false);
                                }
                            },

                            // Category icon
                            span { class: "text-sm", "{entry.category.icon()}" }

                            // Label + sublabel
                            div { class: "flex-1 min-w-0",
                                div { class: "text-sm text-white truncate", "{entry.label}" }
                                div { class: "text-xs text-gray-500 truncate font-mono", "{entry.sublabel}" }
                            }

                            // Category badge
                            span { class: "text-[10px] px-2 py-0.5 rounded border font-medium {entry.category.badge_class()}",
                                "{entry.category.label()}"
                            }
                        }
                    }
                }

                // Footer hint
                div { class: "px-5 py-2.5 border-t border-white/[0.06] flex items-center justify-between text-[10px] text-gray-600",
                    span { "↑↓ Navigate · ↵ Open · Esc Close" }
                    span { class: "font-mono", "⌘K" }
                }
            }
        }
    }
}
