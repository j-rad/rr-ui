use crate::models::GeneralResponse;
use crate::repositories::balancer::BalancerRepository;
use crate::AppState;
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Balancer {
    pub id: String,
    pub tag: String,
    pub selector: Vec<String>, // Tags to balance [ "out-1", "out-2" ]
    pub strategy: String,      // "random" | "leastPing"
}

#[get("/list")]
pub async fn list(data: web::Data<AppState>) -> impl Responder {
    match data.balancer_repo.list().await {
        Ok(list) => HttpResponse::Ok().json(GeneralResponse::success(
            "success",
            Some(serde_json::to_value(list).unwrap()),
        )),
        Err(_) => HttpResponse::Ok().json(GeneralResponse::success(
            "success",
            Some(serde_json::json!([])),
        )),
    }
}

#[post("/add")]
pub async fn add(data: web::Data<AppState>, balancer: web::Json<Balancer>) -> impl Responder {
    let mut new_b = balancer.into_inner();
    if new_b.id.is_empty() {
        new_b.id = chrono::Utc::now().timestamp_millis().to_string();
    }

    match data.balancer_repo.add(new_b).await {
        Ok(_) => HttpResponse::Ok().json(GeneralResponse::success("Balancer added", None)),
        Err(_) => HttpResponse::InternalServerError()
            .json(GeneralResponse::error("Failed to add balancer")),
    }
}

#[post("/del/{id}")]
pub async fn del(data: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    match data.balancer_repo.delete(&id).await {
        Ok(_) => HttpResponse::Ok().json(GeneralResponse::success("Balancer deleted", None)),
        Err(_) => HttpResponse::InternalServerError()
            .json(GeneralResponse::error("Failed to delete balancer")),
    }
}
