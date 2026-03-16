use crate::middleware::ws_telemetry::WsTelemetrySession;
use actix_web::web::Bytes;
use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use tokio::sync::broadcast::Sender;

#[get("/ws/stats")]
pub async fn ws_stats(
    req: HttpRequest,
    stream: web::Payload,
    tx: web::Data<Sender<Bytes>>,
) -> Result<HttpResponse, Error> {
    let rx = tx.subscribe();
    ws::start(WsTelemetrySession::new(rx), &req, stream)
}
