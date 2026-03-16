// src/api/setting.rs
use crate::AppState;
use crate::domain::ports::SettingRepository;
use crate::models::{AllSetting, GeneralResponse};
use crate::rustray_process::SharedRustRayProcess;
use actix_web::{HttpResponse, Responder, get, post, web};

#[get("/all")]
pub async fn get_all_settings(data: web::Data<AppState>) -> impl Responder {
    // Fetch the single settings record (we assume ID 'global')
    // Fetch the single settings record (we assume ID 'global')
    let result = data.setting_repo.get().await;

    match result {
        Ok(Some(settings)) => HttpResponse::Ok().json(GeneralResponse::success(
            "success",
            Some(serde_json::to_value(settings).unwrap()),
        )),
        Ok(None) => {
            // Return defaults if not initialized
            // In a real app, this should probably init the DB on startup
            HttpResponse::Ok().json(GeneralResponse::error("Settings not initialized"))
        }
        Err(e) => HttpResponse::InternalServerError()
            .json(GeneralResponse::error(&format!("DB Error: {}", e))),
    }
}

#[post("/update")]
pub async fn update_setting(
    data: web::Data<AppState>,
    process: web::Data<SharedRustRayProcess>,
    settings: web::Json<AllSetting>,
) -> impl Responder {
    let new_settings = settings.into_inner();

    // Update logic
    // Update logic
    let result = data.setting_repo.save(new_settings).await;

    match result {
        Ok(_) => {
            // Trigger restart to apply new core/path settings
            let mut proc = process.process.lock().await;
            if let Err(e) = proc.restart(&data.db).await {
                return HttpResponse::InternalServerError().json(GeneralResponse::error(&format!(
                    "Settings saved but restart failed: {}",
                    e
                )));
            }
            HttpResponse::Ok().json(GeneralResponse::success(
                "Settings updated and Core restarted",
                None,
            ))
        }
        Err(e) => HttpResponse::InternalServerError()
            .json(GeneralResponse::error(&format!("Failed to update: {}", e))),
    }
}
