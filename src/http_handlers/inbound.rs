// src/http_handlers/inbound.rs - Web Adapters for Inbound API
//
// Thin adapters that extract HTTP request data, call domain services,
// and format HTTP responses. NO business logic here.

use crate::domain::errors::DomainError;
use crate::domain::services::InboundService;
use crate::models::{GeneralResponse, Inbound};
use actix_web::{web, HttpResponse, Responder};
use serde_json::Value;
use std::sync::Arc;

/// Application state for inbound handlers
pub struct InboundHandlerState<
    R: crate::domain::ports::InboundRepository,
    O: crate::domain::ports::OutboundRepository,
> {
    pub service: Arc<InboundService<R, O>>,
    pub validator: Arc<dyn crate::domain::ports::ConfigValidator>,
}

/// List all inbounds
pub async fn list<R, O>(state: web::Data<InboundHandlerState<R, O>>) -> impl Responder
where
    R: crate::domain::ports::InboundRepository + 'static,
    O: crate::domain::ports::OutboundRepository + 'static,
{
    match state.service.list_all().await {
        Ok(inbounds) => {
            let value = serde_json::to_value(inbounds).unwrap_or(Value::Null);
            HttpResponse::Ok().json(GeneralResponse::success("success", Some(value)))
        }
        Err(e) => map_error_to_response(e),
    }
}

/// Add a new inbound
pub async fn add<R, O>(
    state: web::Data<InboundHandlerState<R, O>>,
    inbound: web::Json<Inbound<'static>>,
) -> impl Responder
where
    R: crate::domain::ports::InboundRepository + 'static,
    O: crate::domain::ports::OutboundRepository + 'static,
{
    let inbound = inbound.into_inner();

    match state.service.create(inbound, &*state.validator).await {
        Ok(created) => {
            let value = serde_json::to_value(created).unwrap_or(Value::Null);
            HttpResponse::Ok().json(GeneralResponse::success("Inbound added", Some(value)))
        }
        Err(e) => map_error_to_response(e),
    }
}

/// Update an existing inbound
pub async fn update<R, O>(
    state: web::Data<InboundHandlerState<R, O>>,
    path: web::Path<String>,
    inbound: web::Json<Inbound<'static>>,
) -> impl Responder
where
    R: crate::domain::ports::InboundRepository + 'static,
    O: crate::domain::ports::OutboundRepository + 'static,
{
    let id = path.into_inner();
    let inbound = inbound.into_inner();

    match state.service.update(&id, inbound, &*state.validator).await {
        Ok(updated) => {
            let value = serde_json::to_value(updated).unwrap_or(Value::Null);
            HttpResponse::Ok().json(GeneralResponse::success("Inbound updated", Some(value)))
        }
        Err(e) => map_error_to_response(e),
    }
}

/// Delete an inbound
pub async fn del<R, O>(
    state: web::Data<InboundHandlerState<R, O>>,
    path: web::Path<String>,
) -> impl Responder
where
    R: crate::domain::ports::InboundRepository + 'static,
    O: crate::domain::ports::OutboundRepository + 'static,
{
    let id = path.into_inner();

    match state.service.delete(&id).await {
        Ok(_) => HttpResponse::Ok().json(GeneralResponse::success("Inbound deleted", None)),
        Err(e) => map_error_to_response(e),
    }
}

/// Get inbound by tag
pub async fn get_by_tag<R, O>(
    state: web::Data<InboundHandlerState<R, O>>,
    path: web::Path<String>,
) -> impl Responder
where
    R: crate::domain::ports::InboundRepository + 'static,
    O: crate::domain::ports::OutboundRepository + 'static,
{
    let tag = path.into_inner();

    match state.service.get_by_tag(&tag).await {
        Ok(inbound) => {
            let value = serde_json::to_value(inbound).unwrap_or(Value::Null);
            HttpResponse::Ok().json(GeneralResponse::success("success", Some(value)))
        }
        Err(e) => map_error_to_response(e),
    }
}

/// Map domain errors to HTTP responses
fn map_error_to_response(error: DomainError) -> HttpResponse {
    match error {
        DomainError::NotFound { .. } => {
            HttpResponse::NotFound().json(GeneralResponse::error(&error.to_string()))
        }
        DomainError::ValidationFailed { .. } => {
            HttpResponse::BadRequest().json(GeneralResponse::error(&error.to_string()))
        }
        DomainError::BusinessRuleViolation { .. } => {
            HttpResponse::BadRequest().json(GeneralResponse::error(&error.to_string()))
        }
        DomainError::Conflict { .. } => {
            HttpResponse::Conflict().json(GeneralResponse::error(&error.to_string()))
        }
        DomainError::ConfigurationError { .. } => {
            HttpResponse::BadRequest().json(GeneralResponse::error(&error.to_string()))
        }
        DomainError::RepositoryError { .. } | DomainError::ExternalServiceError { .. } => {
            HttpResponse::InternalServerError().json(GeneralResponse::error(&error.to_string()))
        }
    }
}

/// Configure inbound routes
pub fn configure_routes<R, O>(cfg: &mut web::ServiceConfig, state: InboundHandlerState<R, O>)
where
    R: crate::domain::ports::InboundRepository + 'static,
    O: crate::domain::ports::OutboundRepository + 'static,
{
    cfg.app_data(web::Data::new(state))
        .route("/list", web::get().to(list::<R, O>))
        .route("/add", web::post().to(add::<R, O>))
        .route("/update/{id}", web::post().to(update::<R, O>))
        .route("/del/{id}", web::post().to(del::<R, O>))
        .route("/get/{tag}", web::get().to(get_by_tag::<R, O>));
}
