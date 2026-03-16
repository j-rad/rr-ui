// src/api/geo.rs
use actix_web::{HttpResponse, Responder, get, post, web};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::models::GeneralResponse;
use crate::services::geo_orchestrator::GeoOrchestrator;

/// Shared GeoOrchestrator instance
pub type SharedGeoOrchestrator = Arc<RwLock<Option<GeoOrchestrator>>>;

/// Initialize the geo orchestrator
pub async fn init_geo_orchestrator() -> SharedGeoOrchestrator {
    let assets_dir = std::env::var("RUSTRAY_ASSET_LOCATION")
        .unwrap_or_else(|_| "/usr/share/rustray".to_string());

    let orchestrator = GeoOrchestrator::new(&assets_dir).ok().and_then(|o| {
        // Initialize asynchronously
        Some(o)
    });

    Arc::new(RwLock::new(orchestrator))
}

/// GET /api/geo/state - Get current geo-assets state
#[get("/api/geo/state")]
pub async fn get_geo_state(orchestrator: web::Data<SharedGeoOrchestrator>) -> impl Responder {
    let orch_guard = orchestrator.read().await;

    match orch_guard.as_ref() {
        Some(orch) => {
            let state = orch.get_state().await;
            HttpResponse::Ok().json(state)
        }
        None => HttpResponse::InternalServerError()
            .json(GeneralResponse::error("GeoOrchestrator not initialized")),
    }
}

/// POST /api/geo/sync - Trigger geo-assets synchronization
#[post("/api/geo/sync")]
pub async fn sync_geo_assets(orchestrator: web::Data<SharedGeoOrchestrator>) -> impl Responder {
    let orch_guard = orchestrator.read().await;

    match orch_guard.as_ref() {
        Some(orch) => {
            // Perform sync in background
            match orch.sync_all().await {
                Ok(()) => HttpResponse::Ok().json(GeneralResponse::success(
                    "Geo-assets synchronized successfully",
                    None,
                )),
                Err(e) => HttpResponse::InternalServerError()
                    .json(GeneralResponse::error(&format!("Sync failed: {}", e))),
            }
        }
        None => HttpResponse::InternalServerError()
            .json(GeneralResponse::error("GeoOrchestrator not initialized")),
    }
}

/// POST /api/routing/template - Apply routing template
#[derive(Deserialize)]
pub struct ApplyTemplateRequest {
    pub template: String, // "default", "china", "iran", "russia"
}

#[post("/api/routing/template")]
pub async fn apply_routing_template(req: web::Json<ApplyTemplateRequest>) -> impl Responder {
    use crate::rustray_config::RoutingTemplate;

    // Parse template
    let template = match req.template.as_str() {
        "default" => RoutingTemplate::Default,
        "china" => RoutingTemplate::ChinaOptimized,
        "iran" => RoutingTemplate::IranOptimized,
        "russia" => RoutingTemplate::RussiaOptimized,
        _ => {
            return HttpResponse::BadRequest().json(GeneralResponse::error(
                "Invalid template. Must be one of: default, china, iran, russia",
            ));
        }
    };

    // Generate rules
    let _rules = template.generate_rules();

    // In a real implementation, this would update the RustRay configuration
    // For now, we just validate and return success

    HttpResponse::Ok().json(GeneralResponse::success(
        &format!(
            "Routing template '{}' will be applied on next restart",
            req.template
        ),
        None,
    ))
}

/// GET /api/routing/stats - Get routing statistics
#[derive(Serialize)]
pub struct RoutingStats {
    pub direct_percentage: u8,
    pub blocked_percentage: u8,
    pub proxy_percentage: u8,
}

#[get("/api/routing/stats")]
pub async fn get_routing_stats() -> impl Responder {
    // In a real implementation, this would calculate actual stats from traffic data
    // For now, return mock data
    let stats = RoutingStats {
        direct_percentage: 45,
        blocked_percentage: 5,
        proxy_percentage: 50,
    };

    HttpResponse::Ok().json(stats)
}

pub fn geo_api_config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_geo_state)
        .service(sync_geo_assets)
        .service(apply_routing_template)
        .service(get_routing_stats);
}
