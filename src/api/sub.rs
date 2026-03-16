// src/api/sub.rs

use crate::AppState;
use crate::domain::ports::{InboundRepository, SettingRepository};
use crate::services::subscription::SubscriptionGenerator;
use actix_web::{HttpResponse, Responder, get, web};

#[get("/sub/{token}")]
pub async fn get_subscription(
    data: web::Data<AppState>,
    token: web::Path<String>,
    req: actix_web::HttpRequest,
) -> impl Responder {
    #[cfg(feature = "server")]
    {
        let settings_result = data.setting_repo.get().await;

        match settings_result {
            Ok(Some(settings)) => {
                if settings.sub_api_token.as_ref().map(|s| s.as_str()) != Some(token.as_str()) {
                    return HttpResponse::Forbidden().body("Invalid token");
                }

                let inbounds = match data.inbound_repo.find_all().await {
                    Ok(inbounds) => inbounds,
                    Err(_) => {
                        return HttpResponse::InternalServerError()
                            .body("Failed to fetch inbounds");
                    }
                };

                // Determine host: fallback to request host
                let host = req.connection_info().host().to_string();

                // Detect Client Type (future enhancement)
                // let user_agent = req.headers().get("User-Agent").and_then(|h| h.to_str().ok()).unwrap_or("");

                // Generate generic base64 subscription
                let encoded_configs = SubscriptionGenerator::generate_links(&inbounds, &host);

                HttpResponse::Ok()
                    .content_type("text/plain")
                    .body(encoded_configs)
            }
            _ => HttpResponse::InternalServerError().body("Settings not found"),
        }
    }

    #[cfg(not(feature = "server"))]
    {
        HttpResponse::NotFound().body("API not available in client mode")
    }
}
