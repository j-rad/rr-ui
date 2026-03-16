// src/api/routing.rs
use crate::models::GeneralResponse;
use crate::repositories::routing::RoutingRepository;
use crate::AppState;
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoutingRule {
    pub id: String,
    pub r#type: String,
    pub domain: Vec<String>,
    pub ip: Vec<String>,
    pub port: String,
    pub outbound_tag: String,
}

#[get("/list")]
pub async fn list_rules(data: web::Data<AppState>) -> impl Responder {
    match data.routing_repo.list().await {
        Ok(rules) => HttpResponse::Ok().json(GeneralResponse::success(
            "success",
            Some(serde_json::to_value(rules).unwrap()),
        )),
        Err(_) => HttpResponse::Ok().json(GeneralResponse::success(
            "success",
            Some(serde_json::json!([])),
        )),
    }
}

#[post("/add")]
pub async fn add_rule(data: web::Data<AppState>, rule: web::Json<RoutingRule>) -> impl Responder {
    let id = chrono::Utc::now().timestamp_millis().to_string();
    let mut new_rule = rule.into_inner();
    new_rule.id = id;

    match data.routing_repo.add(new_rule).await {
        Ok(_) => HttpResponse::Ok().json(GeneralResponse::success("Rule added", None)),
        Err(_) => {
            HttpResponse::InternalServerError().json(GeneralResponse::error("Failed to add rule"))
        }
    }
}

#[post("/del/{id}")]
pub async fn del_rule(data: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    match data.routing_repo.delete(&id).await {
        Ok(_) => HttpResponse::Ok().json(GeneralResponse::success("Rule deleted", None)),
        Err(_) => HttpResponse::InternalServerError()
            .json(GeneralResponse::error("Failed to delete rule")),
    }
}
