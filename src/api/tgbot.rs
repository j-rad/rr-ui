use crate::AppState;
use crate::domain::ports::SettingRepository;
use crate::models::GeneralResponse;
use crate::services::tgbot::send_message;
use actix_web::{HttpResponse, Responder, get, post, web};
use serde::Deserialize;
use serde_json::json;

/// Represents the configuration for the Telegram bot.
#[derive(Deserialize)]
pub struct TgConfig {
    /// The Telegram bot token.
    pub token: String,
    /// The chat ID to send messages to.
    pub chat_id: String,
    /// Whether the bot is enabled.
    pub enabled: bool,
    /// Whether to notify on login.
    pub notify_login: bool,
}

/// Handles the GET request to fetch the Telegram bot configuration.
///
/// # Arguments
///
/// * `_data` - The application state.
#[get("/get")]
pub async fn get_config(data: web::Data<AppState>) -> impl Responder {
    let settings = data
        .setting_repo
        .get()
        .await
        .unwrap_or(None)
        .unwrap_or_default();

    let config = json!({
        "token": settings.tg_bot_token.unwrap_or_default(),
        "chat_id": settings.tg_bot_chat_id.unwrap_or_default(),
        "enabled": settings.tg_bot_enable,
        "notify_login": settings.tg_notify_login,
        "notify_expiry": settings.tg_notify_expiry,
        "notify_traffic": settings.tg_notify_traffic
    });
    HttpResponse::Ok().json(GeneralResponse::success("success", Some(config)))
}

#[post("/update")]
pub async fn update_config(
    data: web::Data<AppState>,
    config: web::Json<TgConfig>,
) -> impl Responder {
    let mut settings = data
        .setting_repo
        .get()
        .await
        .unwrap_or(None)
        .unwrap_or_default();

    settings.tg_bot_token = Some(config.token.clone());
    settings.tg_bot_chat_id = Some(config.chat_id.clone());
    settings.tg_bot_enable = config.enabled;
    settings.tg_notify_login = config.notify_login;

    match data.setting_repo.save(settings).await {
        Ok(_) => {
            HttpResponse::Ok().json(GeneralResponse::success("Telegram settings updated", None))
        }
        Err(e) => HttpResponse::InternalServerError().json(GeneralResponse::error(&format!(
            "Failed to update settings: {}",
            e
        ))),
    }
}

/// Handles the POST request to send a test message using the bot.
///
/// # Arguments
///
/// * `config` - The bot configuration from the request body.
#[post("/test")]
pub async fn test_bot(config: web::Json<TgConfig>) -> impl Responder {
    match send_message(
        &config.token,
        &config.chat_id,
        "Test message from X-UI Panel",
    )
    .await
    {
        Ok(_) => {
            HttpResponse::Ok().json(GeneralResponse::success("Message sent successfully", None))
        }
        Err(e) => {
            HttpResponse::Ok().json(GeneralResponse::error(&format!("Failed to send: {}", e)))
        }
    }
}
