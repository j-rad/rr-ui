pub use crate::models::Claims;
use anyhow::Result;
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode};
use totp_rs::{Algorithm, TOTP};

pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    Ok(password_hash.to_string())
}

pub fn verify_password(hash: &str, password: &str) -> bool {
    let parsed_hash = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

pub fn create_jwt(username: &str) -> Result<String> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: username.to_owned(),
        exp: expiration as usize,
    };

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| anyhow::anyhow!(e.to_string()))
}

pub fn verify_jwt(token: &str) -> Result<TokenData<Claims>> {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| anyhow::anyhow!(e.to_string()))
}

pub fn generate_mfa_secret() -> Result<(String, String)> {
    use rand::Rng;
    let raw_secret: [u8; 20] = rand::thread_rng().r#gen();
    let secret_vec = raw_secret.to_vec();
    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret_vec.clone(),
        Some("rr-ui".to_string()),
        "admin".to_string(),
    )
    .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    let secret_b32 = totp.get_secret_base32();
    let totp_uri = totp.get_url();
    Ok((secret_b32, totp_uri))
}

pub fn verify_mfa_code(secret_b32: &str, code: &str) -> bool {
    // Decode base32 secret back to bytes
    let secret = match base32::decode(base32::Alphabet::Rfc4648 { padding: false }, secret_b32) {
        Some(s) => s,
        None => return false,
    };

    let totp = match TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret,
        Some("rr-ui".to_string()),
        "admin".to_string(),
    ) {
        Ok(t) => t,
        Err(_) => return false,
    };

    totp.check_current(code).unwrap_or(false)
}

pub async fn check_ldap_auth(
    server_url: &str,
    base_dn: &str,
    username: &str,
    _password: &str,
) -> bool {
    log::info!(
        "Attempting LDAP auth for user '{}' at '{}' with base DN '{}'",
        username,
        server_url,
        base_dn
    );
    // Placeholder: In a real scenario, you would use an LDAP client library
    // to connect and attempt a bind operation.
    // As external networking is restricted, we'll just log and return false.
    false
}
