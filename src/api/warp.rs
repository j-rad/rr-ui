// src/api/warp.rs
use crate::AppState;
use crate::domain::ports::SettingRepository;
use crate::models::GeneralResponse;
use actix_web::{HttpResponse, Responder, delete, get, post, web};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct WarpRegisterPayload {
    pub license_key: String,
}

#[get("/status")]
pub async fn status(data: web::Data<AppState>) -> impl Responder {
    #[cfg(feature = "server")]
    {
        if let Ok(Some(settings)) = data.setting_repo.get().await {
            let status = if settings.warp_license_key.is_some() {
                "Registered"
            } else {
                "Not Registered"
            };
            return HttpResponse::Ok().json(GeneralResponse::success(
                "WARP info fetched",
                Some(json!({
                    "status": status,
                    "license_key": settings.warp_license_key.as_ref().map(|s| s.as_str()).unwrap_or(""),
                })),
            ));
        }
    }
    HttpResponse::InternalServerError().json(GeneralResponse::error("Failed to fetch WARP info"))
}

#[post("/register")]
pub async fn register(
    data: web::Data<AppState>,
    payload: web::Json<WarpRegisterPayload>,
) -> impl Responder {
    #[cfg(feature = "server")]
    {
        if let Ok(Some(mut settings)) = data.setting_repo.get().await {
            settings.warp_license_key = Some(payload.license_key.clone());
            let result = data.setting_repo.save(settings).await;
            if result.is_ok() {
                return HttpResponse::Ok()
                    .json(GeneralResponse::success("WARP license key saved", None));
            }
        }
    }
    HttpResponse::InternalServerError()
        .json(GeneralResponse::error("Failed to save WARP license key"))
}

#[delete("/panel/warp/delete")]
pub async fn delete_warp_license(data: web::Data<AppState>) -> impl Responder {
    #[cfg(feature = "server")]
    {
        if let Ok(Some(mut settings)) = data.setting_repo.get().await {
            settings.warp_license_key = None;
            if data.setting_repo.save(settings).await.is_ok() {
                return HttpResponse::Ok()
                    .json(GeneralResponse::success("WARP license key deleted", None));
            }
        }
    }
    HttpResponse::InternalServerError()
        .json(GeneralResponse::error("Failed to delete WARP license key"))
}
