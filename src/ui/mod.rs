//! Dioxus UI Module
//!
//! This module contains the Dioxus-based frontend components for the rr-ui admin panel.

pub mod components;

pub mod pages;

pub mod state;

pub mod app;

pub mod server_fns;

pub mod theme;

#[cfg(feature = "server")]
use rust_embed::RustEmbed;

#[cfg(feature = "server")]
#[derive(RustEmbed)]
#[folder = "dist"]
pub struct WebStatic;

#[cfg(feature = "server")]
pub async fn serve_embedded_file(path: actix_web::web::Path<String>) -> actix_web::HttpResponse {
    let path = path.into_inner();
    let path = if path.is_empty() { "index.html" } else { &path };

    match WebStatic::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            actix_web::HttpResponse::Ok()
                .content_type(mime.as_ref())
                .body(content.data.into_owned())
        }
        None => {
            // SPA Fallback
            if let Some(index) = WebStatic::get("index.html") {
                actix_web::HttpResponse::Ok()
                    .content_type("text/html")
                    .body(index.data.into_owned())
            } else {
                actix_web::HttpResponse::NotFound().body("404 Not Found")
            }
        }
    }
}
pub mod sleep;
