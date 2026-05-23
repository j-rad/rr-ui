use crate::AppState;
use crate::models::GeneralResponse;
use crate::rustray_process::SharedRustRayProcess;
use actix_web::{HttpResponse, Responder, get, post, web};
use std::process::Command;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

pub mod proto {
    tonic::include_proto!("rustray.core.app.proxyman.command");
}

use proto::control_service_server::ControlService;
pub use proto::{ConfigEvent, SubscribeConfigRequest, TelemetryReport, TelemetryResponse};

pub struct ControlServiceImpl {
    pub app_state: Arc<crate::server::AppState>,
}

#[tonic::async_trait]
impl ControlService for ControlServiceImpl {
    type SubscribeConfigStream = ReceiverStream<Result<ConfigEvent, Status>>;

    async fn subscribe_config(
        &self,
        request: Request<tonic::Streaming<SubscribeConfigRequest>>,
    ) -> Result<Response<Self::SubscribeConfigStream>, Status> {
        let mut stream = request.into_inner();
        let (tx, rx) = mpsc::channel(128);

        let app_state = self.app_state.clone();

        tokio::spawn(async move {
            let mut last_node_id = None;
            while let Ok(Some(req)) = stream.message().await {
                let node_id = req.node_id;
                last_node_id = Some(node_id.clone());
                log::info!(
                    "Node '{}' subscribed to config updates (bi-directional stream)",
                    node_id
                );
                app_state.node_streams.insert(node_id.clone(), tx.clone());
            }
            if let Some(node_id) = last_node_id {
                log::info!("Node '{}' disconnected, cleaning up config stream", node_id);
                app_state.node_streams.remove(&node_id);
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn push_telemetry(
        &self,
        request: Request<TelemetryReport>,
    ) -> Result<Response<TelemetryResponse>, Status> {
        let report = request.into_inner();

        // Record traffic for billing delta sync
        for traffic in report.traffic {
            self.app_state.billing_sync.record_traffic(
                traffic.inbound_tag,
                traffic.email,
                traffic.up,
                traffic.down,
            );
        }

        // Potential: Update node health in mesh orchestrator
        // self.app_state.mesh_orchestrator.heartbeat(&report.node_id, report.system).await;

        Ok(Response::new(TelemetryResponse { success: true }))
    }
}

#[get("/status")]
pub async fn status(data: web::Data<SharedRustRayProcess>) -> impl Responder {
    let mut process = data.process.lock().await;
    let running = process.is_running();

    // Fetch real version via `rustray --version`
    // Assuming rustray binary is in path or known location. process.bin_path
    let version = if let Ok(output) = Command::new("rustray").arg("--version").output() {
        String::from_utf8_lossy(&output.stdout)
            .split_whitespace()
            .nth(1)
            .unwrap_or("Unknown")
            .to_string()
    } else {
        "Unknown".to_string()
    };

    let response = serde_json::json!({
        "running": running,
        "version": version
    });

    HttpResponse::Ok().json(GeneralResponse::success("success", Some(response)))
}

#[post("/restartRustRayService")]
pub async fn restart(
    data: web::Data<SharedRustRayProcess>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let mut process = data.process.lock().await;
    // We need to pass DB client to restart to regenerate config
    match process.restart(&app_state.db).await {
        Ok(_) => HttpResponse::Ok().json(GeneralResponse::success(
            "RustRay restarted successfully",
            None,
        )),
        Err(e) => HttpResponse::InternalServerError()
            .json(GeneralResponse::error(&format!("Failed to restart: {}", e))),
    }
}

#[post("/stopRustRayService")]
pub async fn stop(data: web::Data<SharedRustRayProcess>) -> impl Responder {
    let mut process = data.process.lock().await;
    match process.stop() {
        Ok(_) => HttpResponse::Ok().json(GeneralResponse::success("RustRay stopped", None)),
        Err(e) => HttpResponse::InternalServerError()
            .json(GeneralResponse::error(&format!("Failed to stop: {}", e))),
    }
}

#[get("/getRustRayVersion")]
pub async fn get_rustray_version() -> impl Responder {
    // Query actual rustray version from the binary
    let current_version = if let Ok(output) = Command::new("rustray").arg("--version").output() {
        String::from_utf8_lossy(&output.stdout)
            .split_whitespace()
            .nth(1)
            .unwrap_or("Unknown")
            .to_string()
    } else {
        "Unknown".to_string()
    };

    // Return current version and recent stable versions
    HttpResponse::Ok().json(GeneralResponse::success(
        "success",
        Some(serde_json::json!({
            "current": current_version,
            "available": ["1.0.0", "1.1.0"]
        })),
    ))
}

#[post("/installRustRay/{version}")]
pub async fn install_rustray(path: web::Path<String>) -> impl Responder {
    let _version = path.into_inner();
    HttpResponse::NotImplemented().json(GeneralResponse::error(
        "Auto-installation of RustRay is not yet supported via API",
    ))
}
