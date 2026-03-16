//! Secret Generator Component
//!
//! Client-side utility for generating cryptographic secrets:
//! - UUID v4 for client IDs
//! - Reality keypairs (X25519)
//! - Short IDs for Reality protocol

use dioxus::prelude::*;

/// Generate a new UUID v4 string
pub fn generate_uuid_v4() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: [u8; 16] = rng.r#gen();
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
        (bytes[6] & 0x0f) | 0x40,
        bytes[7], // Version 4
        (bytes[8] & 0x3f) | 0x80,
        bytes[9], // Variant 1
        bytes[10],
        bytes[11],
        bytes[12],
        bytes[13],
        bytes[14],
        bytes[15]
    )
}

/// Generate a random short ID (hex string, 1-8 bytes)
pub fn generate_short_id(length: usize) -> String {
    use rand::Rng;
    let length = length.clamp(1, 16);
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..length).map(|_| rng.r#gen()).collect();
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Generate a Reality keypair (X25519)
/// Returns (private_key_base64, public_key_base64)
/// Generate a Reality keypair (X25519)
/// Returns (private_key_base64, public_key_base64)
pub fn generate_reality_keypair() -> (String, String) {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use rand::rngs::OsRng;
    use x25519_dalek::{PublicKey, StaticSecret};

    // Generate valid X25519 keypair using system randomness (or browser crypto via getrandom)
    let secret = StaticSecret::random_from_rng(OsRng);
    let public = PublicKey::from(&secret);

    (
        STANDARD.encode(secret.to_bytes()),
        STANDARD.encode(public.to_bytes()),
    )
}

#[derive(Props, Clone, PartialEq)]
pub struct SecretGeneratorProps {
    /// Type of secret to generate
    #[props(default = SecretType::Uuid)]
    pub secret_type: SecretType,
    /// Label for the input
    #[props(default)]
    pub label: Option<String>,
    /// Signal to store the generated value
    pub value: Signal<String>,
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum SecretType {
    #[default]
    Uuid,
    ShortId,
    RealityPrivateKey,
    RealityPublicKey,
}

impl SecretType {
    fn placeholder(&self) -> &'static str {
        match self {
            Self::Uuid => "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
            Self::ShortId => "abcd1234",
            Self::RealityPrivateKey => "Base64 encoded private key",
            Self::RealityPublicKey => "Base64 encoded public key",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            Self::Uuid => "fingerprint",
            Self::ShortId => "tag",
            Self::RealityPrivateKey => "key",
            Self::RealityPublicKey => "vpn_key",
        }
    }
}

/// Secret generator input with copy and regenerate buttons
#[component]
pub fn SecretGenerator(props: SecretGeneratorProps) -> Element {
    let mut value = props.value;
    let mut copied = use_signal(|| false);

    let generate = {
        let secret_type = props.secret_type;
        move |_| {
            let new_value = match secret_type {
                SecretType::Uuid => generate_uuid_v4(),
                SecretType::ShortId => generate_short_id(8),
                SecretType::RealityPrivateKey => generate_reality_keypair().0,
                SecretType::RealityPublicKey => generate_reality_keypair().1,
            };
            value.set(new_value);
            copied.set(false);
        }
    };

    let copy_to_clipboard = move |_| {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            if let Some(window) = web_sys::window() {
                let navigator = window.navigator();
                let clipboard = navigator.clipboard();
                let val = value();
                wasm_bindgen_futures::spawn_local(async move {
                    let _ = clipboard.write_text(&val);
                });
                copied.set(true);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            copied.set(true);
        }
    };

    let icon = props.secret_type.icon();
    let placeholder = props.secret_type.placeholder();

    rsx! {
        div { class: "space-y-1",
            if let Some(ref label) = props.label {
                label { class: "block text-sm font-medium text-gray-300", "{label}" }
            }
            div { class: "flex gap-2",
                div { class: "relative flex-1",
                    span { class: "absolute left-3 top-1/2 -translate-y-1/2 material-symbols-outlined text-gray-500 text-[18px]",
                        "{icon}"
                    }
                    input {
                        r#type: "text",
                        class: "w-full h-10 pl-10 pr-3 bg-bg-tertiary border border-border rounded-lg text-white text-sm font-mono focus:border-primary focus:outline-none",
                        value: "{value}",
                        placeholder: "{placeholder}",
                        readonly: true,
                    }
                }
                button {
                    class: "h-10 px-3 bg-bg-tertiary border border-border rounded-lg text-gray-400 hover:text-white hover:border-primary transition-colors",
                    onclick: generate,
                    title: "Generate new",
                    span { class: "material-symbols-outlined text-[18px]", "refresh" }
                }
                button {
                    class: "h-10 px-3 bg-bg-tertiary border border-border rounded-lg transition-colors",
                    class: if copied() { "text-green-400 border-green-400/50" } else { "text-gray-400 hover:text-white hover:border-primary" },
                    onclick: copy_to_clipboard,
                    title: "Copy to clipboard",
                    if copied() {
                        span { class: "material-symbols-outlined text-[18px]", "check" }
                    } else {
                        span { class: "material-symbols-outlined text-[18px]", "content_copy" }
                    }
                }
            }
        }
    }
}

/// Reality keypair generator component
#[derive(Props, Clone, PartialEq)]
pub struct RealityKeyGeneratorProps {
    pub private_key: Signal<String>,
    pub public_key: Signal<String>,
}

#[component]
pub fn RealityKeyGenerator(props: RealityKeyGeneratorProps) -> Element {
    let mut private_key = props.private_key;
    let mut public_key = props.public_key;

    let generate_pair = move |_| {
        let (priv_key, pub_key) = generate_reality_keypair();
        private_key.set(priv_key);
        public_key.set(pub_key);
    };

    rsx! {
        div { class: "space-y-4",
            div { class: "flex items-center justify-between",
                span { class: "text-sm font-medium text-gray-300", "Reality Keys" }
                button {
                    class: "flex items-center gap-2 px-3 py-1.5 text-sm bg-primary/10 text-primary rounded-lg hover:bg-primary/20 transition-colors",
                    onclick: generate_pair,
                    span { class: "material-symbols-outlined text-[16px]", "autorenew" }
                    "Generate Pair"
                }
            }
            SecretGenerator {
                label: "Private Key".to_string(),
                secret_type: SecretType::RealityPrivateKey,
                value: private_key,
            }
            SecretGenerator {
                label: "Public Key".to_string(),
                secret_type: SecretType::RealityPublicKey,
                value: public_key,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_uuid_v4() {
        let uuid = generate_uuid_v4();
        assert_eq!(uuid.len(), 36);
        assert!(uuid.contains('-'));
    }

    #[test]
    fn test_generate_short_id() {
        let short_id = generate_short_id(8);
        assert_eq!(short_id.len(), 16); // 8 bytes = 16 hex chars
    }

    #[test]
    fn test_generate_reality_keypair() {
        let (priv_key, pub_key) = generate_reality_keypair();
        assert!(!priv_key.is_empty());
        assert!(!pub_key.is_empty());
        // Base64 encoded 32 bytes should be 44 chars
        assert_eq!(priv_key.len(), 44);
        assert_eq!(pub_key.len(), 44);
    }
}
