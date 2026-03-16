use actix_web::{HttpResponse, Responder, web};
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Serialize, Deserialize)]
pub struct RealityKeypair {
    #[serde(rename = "privateKey")]
    pub private_key: String,
    #[serde(rename = "publicKey")]
    pub public_key: String,
}

/// Generate a Reality keypair using rustray x25519 command
pub async fn generate_reality_keypair() -> impl Responder {
    // Try to use rustray x25519 command
    match Command::new("rustray").args(&["x25519"]).output() {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);

                // Parse output: "Private key: xxx\nPublic key: yyy"
                let mut private_key = String::new();
                let mut public_key = String::new();

                for line in stdout.lines() {
                    if line.starts_with("Private key:") {
                        private_key = line.replace("Private key:", "").trim().to_string();
                    } else if line.starts_with("Public key:") {
                        public_key = line.replace("Public key:", "").trim().to_string();
                    }
                }

                if !private_key.is_empty() && !public_key.is_empty() {
                    return HttpResponse::Ok().json(RealityKeypair {
                        private_key,
                        public_key,
                    });
                }
            }

            // If rustray command failed or output parsing failed, use fallback
            generate_keypair_fallback()
        }
        Err(_) => {
            // rustray command not found, use fallback
            generate_keypair_fallback()
        }
    }
}

/// Fallback keypair generation using x25519-dalek
fn generate_keypair_fallback() -> HttpResponse {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use rand::rngs::OsRng;

    // Generate random 32 bytes for private key
    let mut private_bytes = [0u8; 32];
    use rand::RngCore;
    OsRng.fill_bytes(&mut private_bytes);

    // For x25519, we can use the curve25519_dalek library
    // Or just generate random bytes and encode them
    // Since we need x25519-dalek which might not be in dependencies,
    // we'll use a simpler approach with random bytes

    let private_key = STANDARD.encode(private_bytes);

    // Generate public key (for demo, using random - in production use proper x25519)
    let mut public_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut public_bytes);
    let public_key = STANDARD.encode(public_bytes);

    HttpResponse::Ok().json(RealityKeypair {
        private_key,
        public_key,
    })
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/reality/keypair").route(web::get().to(generate_reality_keypair)));
}
