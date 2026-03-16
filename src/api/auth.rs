// src/api/auth.rs
use crate::AppState;
use crate::domain::ports::SettingRepository;
use crate::models::{AllSetting, GeneralResponse, LoginPayload};
use crate::services::auth::{
    check_ldap_auth, create_jwt, generate_mfa_secret, hash_password, verify_mfa_code,
    verify_password,
};
use actix_web::{HttpResponse, Responder, get, post, web};
use serde::Deserialize;
use serde_json::json;

/// Represents the payload for verifying an MFA code.
#[derive(Deserialize)]
pub struct MfaVerifyPayload {
    /// The MFA code from the user's authenticator app.
    pub code: String,
}

/// Handles the POST request for user login.
///
/// This function implements the complete authentication flow:
/// 1. Checks for LDAP authentication if it's configured.
/// 2. If LDAP is not used or fails, it falls back to local database authentication.
/// 3. If authentication is successful, it checks for a Multi-Factor Authentication (MFA) code if enabled.
/// 4. On the first run (no settings in DB), it allows login with "admin"/"admin" and creates default settings.
/// 5. If all checks pass, it generates and returns a JWT.
///
/// # Arguments
///
/// * `data` - The application state.
/// * `payload` - The login credentials from the request body.
#[post("/login")]
pub async fn login(
    req: actix_web::HttpRequest,
    data: web::Data<AppState>,
    payload: web::Json<LoginPayload>,
) -> impl Responder {
    #[cfg(feature = "server")]
    {
        let settings_result = data.setting_repo.get().await;

        match settings_result {
            Ok(Some(settings)) => {
                let mut is_auth_ok = false;

                // 1. Check LDAP if configured
                if let (Some(url), Some(dn)) = (
                    settings.ldap_server_url.as_ref(),
                    settings.ldap_base_dn.as_ref(),
                ) {
                    if !url.is_empty() && !dn.is_empty() {
                        if check_ldap_auth(url, dn, &payload.username, &payload.password).await {
                            is_auth_ok = true;
                        }
                    }
                }

                // 2. If LDAP fails or not configured, check local DB
                if !is_auth_ok {
                    if settings.username == payload.username
                        && verify_password(&settings.password_hash, &payload.password)
                    {
                        is_auth_ok = true;
                    }
                }

                if !is_auth_ok {
                    if let Some(addr) = req.peer_addr() {
                        data.shield.record_failure(addr.ip());
                    }
                    return HttpResponse::Ok()
                        .json(GeneralResponse::error("Invalid username or password"));
                }

                // 3. Check MFA if enabled
                if settings.is_two_factor_enabled {
                    if payload.two_factor_code.is_none() {
                        return HttpResponse::Ok()
                            .json(GeneralResponse::success("MFA_REQUIRED", None));
                    }

                    let secret_opt = settings.two_factor_secret.as_ref();
                    let secret: &str = match secret_opt {
                        Some(s) => s,
                        None => "",
                    };
                    let code = payload.two_factor_code.as_deref().unwrap_or_default();
                    if !verify_mfa_code(secret, code) {
                        if let Some(addr) = req.peer_addr() {
                            data.shield.record_failure(addr.ip());
                        }
                        return HttpResponse::Ok().json(GeneralResponse::error("Invalid MFA code"));
                    }
                }

                // Auth successful
                match create_jwt(&payload.username) {
                    Ok(token) => HttpResponse::Ok().json(GeneralResponse::success(
                        "Login successful",
                        Some(json!({ "token": token })),
                    )),
                    Err(_) => HttpResponse::InternalServerError()
                        .json(GeneralResponse::error("Failed to generate token")),
                }
            }
            Ok(None) => {
                // No settings found, handle initial setup
                if payload.username == "admin" && payload.password == "admin" {
                    if let Ok(hash) = hash_password("admin") {
                        let default_settings = AllSetting {
                            username: "admin".to_string(),
                            password_hash: hash,
                            ..Default::default()
                        };
                        let _ = data.setting_repo.save(default_settings).await;
                    }
                    match create_jwt(&payload.username) {
                        Ok(token) => HttpResponse::Ok().json(GeneralResponse::success(
                            "Login successful",
                            Some(json!({ "token": token })),
                        )),
                        Err(_) => HttpResponse::InternalServerError()
                            .json(GeneralResponse::error("Failed to generate token")),
                    }
                } else {
                    HttpResponse::Ok().json(GeneralResponse::error(
                        "Invalid username or password (default)",
                    ))
                }
            }
            Err(e) => {
                // DB Error is not necessarily an auth failure, but let's be safe?
                // Probably don't ban for DB errors.
                HttpResponse::InternalServerError()
                    .json(GeneralResponse::error(&format!("DB Error: {}", e)))
            }
        }
    }

    #[cfg(not(feature = "server"))]
    {
        // Client mode auth - simplified
        if payload.username == "admin" && payload.password == "admin" {
            match create_jwt(&payload.username) {
                Ok(token) => HttpResponse::Ok().json(GeneralResponse::success(
                    "Login successful",
                    Some(json!({ "token": token })),
                )),
                Err(_) => HttpResponse::InternalServerError()
                    .json(GeneralResponse::error("Failed to generate token")),
            }
        } else {
            if let Some(addr) = req.peer_addr() {
                data.shield.record_failure(addr.ip());
            }
            HttpResponse::Ok().json(GeneralResponse::error("Invalid username or password"))
        }
    }
}

/// Handles the GET request to generate a new MFA secret.
///
/// Returns a new secret key and a data URI for the QR code.
///
/// # Arguments
///
/// * `data` - The application state.
#[get("/mfa/generate")]
pub async fn get_mfa_secret(_data: web::Data<AppState>) -> impl Responder {
    match generate_mfa_secret() {
        Ok((secret, uri)) => HttpResponse::Ok().json(GeneralResponse::success(
            "MFA secret generated",
            Some(json!({ "secret": secret, "uri": uri })),
        )),
        Err(_) => HttpResponse::InternalServerError()
            .json(GeneralResponse::error("Failed to generate MFA secret")),
    }
}

/// Handles the POST request to verify and enable MFA.
///
/// Verifies the provided MFA code against the secret stored in the settings.
/// If successful, it enables MFA for the user and saves the setting.
///
/// # Arguments
///
/// * `data` - The application state.
/// * `payload` - The MFA code to verify.
#[post("/mfa/verify")]
pub async fn verify_mfa(
    data: web::Data<AppState>,
    payload: web::Json<MfaVerifyPayload>,
) -> impl Responder {
    #[cfg(feature = "server")]
    {
        let settings_result = data.setting_repo.get().await;
        match settings_result {
            Ok(Some(mut settings)) => {
                if let Some(secret) = settings.two_factor_secret.as_ref() {
                    let secret_str: &str = secret;
                    if verify_mfa_code(secret_str, &payload.code) {
                        settings.is_two_factor_enabled = true;
                        let save_result: crate::domain::errors::DomainResult<()> =
                            data.setting_repo.save(settings).await;
                        if save_result.is_ok() {
                            HttpResponse::Ok()
                                .json(GeneralResponse::success("MFA enabled successfully", None))
                        } else {
                            HttpResponse::InternalServerError()
                                .json(GeneralResponse::error("Failed to save settings"))
                        }
                    } else {
                        HttpResponse::Ok().json(GeneralResponse::error("Invalid MFA code"))
                    }
                } else {
                    HttpResponse::BadRequest().json(GeneralResponse::error("MFA secret not set"))
                }
            }
            _ => HttpResponse::InternalServerError()
                .json(GeneralResponse::error("Failed to get settings")),
        }
    }
    #[cfg(not(feature = "server"))]
    {
        HttpResponse::Ok().json(GeneralResponse::error("MFA not supported in client mode"))
    }
}
