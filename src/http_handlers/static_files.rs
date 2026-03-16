use crate::AppState;
use crate::models::AllSetting;
use crate::repositories::setting::SettingOps;
#[cfg(feature = "server")]
use crate::ui::WebStatic;
use actix_web::{HttpRequest, HttpResponse, http::header::ContentType, web};
use mime_guess::from_path;

pub async fn serve_static_asset(req: HttpRequest, app_data: web::Data<AppState>) -> HttpResponse {
    #[cfg(feature = "server")]
    {
        let path = if req.path() == "/" {
            "index.html"
        } else {
            req.path().trim_start_matches('/')
        };

        match WebStatic::get(path) {
            Some(content) => {
                if path == "index.html" {
                    return serve_index_with_injection(content, &app_data).await;
                }
                let mime_type = from_path(path).first_or_octet_stream();
                HttpResponse::Ok()
                    .content_type(mime_type.as_ref())
                    .body(content.data.into_owned())
            }
            None => {
                // Only apply SPA fallback for routes (no file extension)
                if path.contains('.') {
                    HttpResponse::NotFound().body("404 Not Found")
                } else {
                    if let Some(content) = WebStatic::get("index.html") {
                        return serve_index_with_injection(content, &app_data).await;
                    } else {
                        HttpResponse::NotFound().body("404 Not Found")
                    }
                }
            }
        }
    }
    #[cfg(not(feature = "server"))]
    HttpResponse::NotFound().body("Server feature disabled")
}

#[cfg(feature = "server")]
async fn serve_index_with_injection(
    content: rust_embed::EmbeddedFile,
    app_data: &web::Data<AppState>,
) -> HttpResponse {
    let mime_type = ContentType::html();
    let body_str = match std::str::from_utf8(&content.data) {
        Ok(s) => s.to_string(),
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    // Fetch secret path
    let secret_path = match <AllSetting as SettingOps>::get(&app_data.db).await {
        Ok(Some(settings)) => settings.panel_secret_path,
        _ => "/psb".to_string(),
    };

    log::info!("Injecting base path '{}' into index.html", secret_path);

    let new_base = format!("base: \"{}\"", secret_path);

    let modified_body = if body_str.contains("base: \"\"") {
        body_str.replace("base: \"\"", &new_base)
    } else if body_str.contains("base:\"\"") {
        body_str.replace("base:\"\"", &new_base)
    } else {
        body_str
    };

    HttpResponse::Ok()
        .content_type(mime_type)
        .body(modified_body)
}
