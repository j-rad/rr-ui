// src/api/backup.rs
use crate::models::{ClientTraffic, GeneralResponse, Inbound};
use crate::AppState;
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents the data structure for database backups.
#[derive(Serialize, Deserialize)]
pub struct BackupData {
    /// A list of inbound configurations.
    pub inbounds: Vec<Inbound<'static>>,
    /// A list of client traffic records.
    pub client_traffic: Vec<ClientTraffic>,
    /// A list of general settings.
    pub settings: Vec<Value>,
}

/// Handles the GET request to export the database as a JSON file.
#[get("/export")]
pub async fn export_db(data: web::Data<AppState>) -> impl Responder {
    let inbounds: Vec<Inbound<'static>> =
        data.db.client.select("inbound").await.unwrap_or_default();
    let client_traffic: Vec<ClientTraffic> = data
        .db
        .client
        .select("client_traffic")
        .await
        .unwrap_or_default();
    let settings: Vec<Value> = data.db.client.select("setting").await.unwrap_or_default();

    let backup = BackupData {
        inbounds,
        client_traffic,
        settings,
    };
    let json = serde_json::to_string_pretty(&backup).unwrap_or_default();

    HttpResponse::Ok()
        .content_type("application/json")
        .append_header((
            "Content-Disposition",
            "attachment; filename=\"x-ui-backup.json\"",
        ))
        .body(json)
}

/// Handles the POST request to import data from a JSON backup file.
#[post("/import")]
pub async fn import_db(data: web::Data<AppState>, body: web::Json<BackupData>) -> impl Responder {
    let _ = data
        .db
        .client
        .query("DELETE inbound; DELETE client_traffic; DELETE setting;")
        .await;

    // Clone data to owned types to satisfy 'static lifetime requirements for async blocks if needed
    // and to avoid using references to the ephemeral body
    let inbounds = body.inbounds.clone();
    let traffic = body.client_traffic.clone();

    for inbound in inbounds {
        if let Some(ref id) = inbound.id {
            // Explicitly use reference to avoid ambiguity issues if any, although Inbound<'static> should work.
            // We specify result type to hint inference.
            let _: Result<Option<Inbound<'static>>, _> = data
                .db
                .client
                .create(("inbound", id.id.to_string()))
                .content(inbound)
                .await;
        } else {
            let _: Result<Option<Inbound<'static>>, _> =
                data.db.client.create("inbound").content(inbound).await;
        }
    }
    for t in traffic {
        let _: Result<Option<ClientTraffic>, _> = data
            .db
            .client
            .create(("client_traffic", t.email.clone()))
            .content(t)
            .await;
    }

    HttpResponse::Ok().json(GeneralResponse::success(
        "Backup restored successfully",
        None,
    ))
}
