//! Login Page
//!
//! Authentication screen with wave background and modern form design.

// Shared types - always available
use crate::ui::server_fns::LoginRequest;
// Server functions - only available with web feature
#[cfg(feature = "web")]
use crate::ui::server_fns::login;
#[allow(unused_imports)]
use crate::ui::state::{GlobalState, NotificationType};
use dioxus::prelude::*;

#[component]
pub fn LoginPage() -> Element {
    let state = use_context::<GlobalState>();
    let navigator = use_navigator();

    let mut username = use_signal(|| "".to_string());
    let mut password = use_signal(|| "".to_string());
    let mut mfa_code = use_signal(|| "".to_string());
    let mut loading = use_signal(|| false);
    let mut error_msg = use_signal(|| "".to_string());
    let mut requires_mfa = use_signal(|| false);

    let handle_login = {
        let state = state.clone();
        move |_| {
            let mut state = state.clone();
            spawn(async move {
                loading.set(true);
                error_msg.set("".to_string());

                let req = LoginRequest {
                    username: username(),
                    password: password(),
                    mfa_code: if requires_mfa() {
                        Some(mfa_code())
                    } else {
                        None
                    },
                };

                // Login is only available with web feature
                #[cfg(feature = "web")]
                {
                    match login(req).await {
                        Ok(resp) => {
                            if resp.success {
                                // Update global state
                                state.is_authenticated.set(true);
                                state.auth_token.set(Some(resp.token));
                                state.push_notification(
                                    "Login successful",
                                    NotificationType::Success,
                                );

                                // Redirect to dashboard
                                navigator.push(crate::ui::app::Route::Dashboard {});
                            } else if resp.requires_mfa {
                                requires_mfa.set(true);
                                // Focus MFA input would be nice here
                                state.push_notification(
                                    "Please enter 2FA code",
                                    NotificationType::Info,
                                );
                            } else {
                                error_msg.set(resp.message);
                                state.push_notification("Login failed", NotificationType::Error);
                            }
                        }
                        Err(e) => {
                            error_msg.set(format!("Connection error: {}", e));
                            state.push_notification("Connection failed", NotificationType::Error);
                        }
                    }
                }

                // Server-only builds don't support login from UI
                #[cfg(not(feature = "web"))]
                {
                    let _ = req; // Suppress unused warning
                    error_msg.set("Login not available in this build".to_string());
                }

                loading.set(false);
            });
        }
    };

    rsx! {
        div { class: "min-h-screen bg-bg flex flex-col items-center justify-center relative overflow-hidden",
            // Animated gradient mesh background
            div { class: "absolute inset-0 z-0",
                // Primary gradient orb
                div {
                    class: "absolute top-1/4 -left-20 w-96 h-96 bg-primary/20 rounded-full blur-3xl animate-float",
                    style: "animation-delay: 0s;"
                }
                // Secondary gradient orb
                div {
                    class: "absolute bottom-1/4 -right-20 w-80 h-80 bg-accent-purple/15 rounded-full blur-3xl animate-float",
                    style: "animation-delay: 1s;"
                }
                // Tertiary gradient orb
                div {
                    class: "absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[600px] bg-cyan-500/5 rounded-full blur-3xl"
                }
                // Grid overlay
                div {
                    class: "absolute inset-0 opacity-[0.02]",
                    style: "background-image: linear-gradient(rgba(255,255,255,0.1) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,0.1) 1px, transparent 1px); background-size: 60px 60px;"
                }
            }

            // Login Container - Glass Card
            div { class: "w-full max-w-sm z-10 p-8",
                div { class: "relative bg-glass-bg/60 backdrop-blur-xl border border-glass-border rounded-2xl p-8 shadow-2xl",
                    // Subtle gradient overlay
                    div { class: "absolute inset-0 bg-gradient-to-br from-white/[0.03] via-transparent to-black/[0.02] pointer-events-none rounded-2xl" }

                    // Content
                    div { class: "relative",
                        // Logo & Header
                        div { class: "text-center mb-8",
                            div { class: "inline-flex p-4 rounded-2xl bg-primary/10 text-primary mb-5 shadow-lg shadow-primary/20",
                                span { class: "material-symbols-outlined text-4xl drop-shadow-[0_0_8px_rgba(0,212,255,0.5)]", "token" }
                            }
                            h1 { class: "text-2xl font-bold bg-gradient-to-r from-text-main to-text-secondary bg-clip-text text-transparent tracking-tight", "Welcome Back" }
                            p { class: "text-text-secondary mt-2.5 text-sm", "Sign in to access your dashboard" }
                        }

                        form {
                            class: "space-y-5",
                            onsubmit: handle_login,

                            // Username
                            div { class: "relative group",
                                span { class: "absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none text-text-muted group-focus-within:text-primary transition-colors",
                                    span { class: "material-symbols-outlined text-[20px]", "person" }
                                }
                                input {
                                    class: "w-full bg-bg-tertiary/50 border border-border text-text-main text-sm rounded-xl block pl-12 p-3.5 focus:ring-2 focus:ring-primary/30 focus:border-primary placeholder-text-muted transition-all duration-200",
                                    placeholder: "Username",
                                    value: "{username}",
                                    required: true,
                                    autofocus: true,
                                    oninput: move |e| username.set(e.value())
                                }
                            }

                            // Password
                            div { class: "relative group",
                                 span { class: "absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none text-text-muted group-focus-within:text-primary transition-colors",
                                    span { class: "material-symbols-outlined text-[20px]", "lock" }
                                }
                                input {
                                    class: "w-full bg-bg-tertiary/50 border border-border text-text-main text-sm rounded-xl block pl-12 p-3.5 focus:ring-2 focus:ring-primary/30 focus:border-primary placeholder-text-muted transition-all duration-200",
                                    placeholder: "Password",
                                    r#type: "password",
                                    value: "{password}",
                                    required: true,
                                    oninput: move |e| password.set(e.value())
                                }
                            }

                            // MFA Code (Conditional)
                            if requires_mfa() {
                                 div { class: "relative group animate-fade-in",
                                     span { class: "absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none text-text-muted group-focus-within:text-primary transition-colors",
                                        span { class: "material-symbols-outlined text-[20px]", "key" }
                                    }
                                    input {
                                        class: "w-full bg-bg-tertiary/50 border border-border text-text-main text-sm rounded-xl block pl-12 p-3.5 focus:ring-2 focus:ring-primary/30 focus:border-primary placeholder-text-muted transition-all duration-200",
                                        placeholder: "2FA Code",
                                        value: "{mfa_code}",
                                        required: true,
                                        oninput: move |e| mfa_code.set(e.value())
                                    }
                                }
                            }

                            // Error Message
                            if !error_msg().is_empty() {
                                div { class: "p-3.5 text-sm text-rose-400 bg-rose-500/10 border border-rose-500/20 rounded-xl flex items-center gap-3 animate-fade-in",
                                    span { class: "material-symbols-outlined text-[18px]", "error" }
                                    "{error_msg}"
                                }
                            }

                            // Submit Button
                            button {
                                class: "w-full text-white bg-gradient-to-r from-primary to-cyan-400 hover:shadow-lg hover:shadow-primary/30 focus:ring-4 focus:outline-none focus:ring-primary/30 font-semibold rounded-xl text-sm px-5 py-3.5 text-center flex justify-center items-center gap-2.5 disabled:opacity-50 disabled:cursor-not-allowed transition-all duration-300",
                                r#type: "submit",
                                disabled: loading(),
                                if loading() {
                                    span { class: "w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin" }
                                    "Signing in..."
                                } else {
                                     span { class: "material-symbols-outlined text-[20px]", "login" }
                                    "Sign In"
                                }
                            }
                        }
                    }
                }

                // Footer
                div { class: "mt-8 text-center text-xs text-text-muted",
                    "RR-UI v0.7.0"
                }
            }
        }
    }
}
