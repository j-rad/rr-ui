// src/http_handlers/user.rs - Web Adapters for User/Client API
//
// Thin web adapters for user/client traffic management

use crate::domain::errors::DomainError;
use crate::domain::services::UserService;
use crate::models::GeneralResponse;
use actix_web::{web, HttpResponse, Responder};
use serde_json::Value;
use std::sync::Arc;

/// Application state for user handlers
pub struct UserHandlerState<
    U: crate::domain::ports::UserRepository,
    I: crate::domain::ports::InboundRepository,
> {
    pub service: Arc<UserService<U, I>>,
}

/// Get traffic for a specific user
pub async fn get_traffic<U, I>(
    state: web::Data<UserHandlerState<U, I>>,
    path: web::Path<String>,
) -> impl Responder
where
    U: crate::domain::ports::UserRepository + 'static,
    I: crate::domain::ports::InboundRepository + 'static,
{
    let email = path.into_inner();

    match state.service.get_traffic(&email).await {
        Ok(traffic) => {
            let value = serde_json::to_value(traffic).unwrap_or(Value::Null);
            HttpResponse::Ok().json(GeneralResponse::success("success", Some(value)))
        }
        Err(e) => map_error_to_response(e),
    }
}

/// Reset traffic for a user
pub async fn reset_traffic<U, I>(
    state: web::Data<UserHandlerState<U, I>>,
    path: web::Path<String>,
) -> impl Responder
where
    U: crate::domain::ports::UserRepository + 'static,
    I: crate::domain::ports::InboundRepository + 'static,
{
    let email = path.into_inner();

    match state.service.reset_traffic(&email).await {
        Ok(_) => HttpResponse::Ok().json(GeneralResponse::success("Traffic reset", None)),
        Err(e) => map_error_to_response(e),
    }
}

/// Get all user traffic
pub async fn get_all_traffic<U, I>(state: web::Data<UserHandlerState<U, I>>) -> impl Responder
where
    U: crate::domain::ports::UserRepository + 'static,
    I: crate::domain::ports::InboundRepository + 'static,
{
    match state.service.get_all_traffic().await {
        Ok(traffic) => {
            let value = serde_json::to_value(traffic).unwrap_or(Value::Null);
            HttpResponse::Ok().json(GeneralResponse::success("success", Some(value)))
        }
        Err(e) => map_error_to_response(e),
    }
}

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

pub fn configure_routes<U, I>(cfg: &mut web::ServiceConfig, state: UserHandlerState<U, I>)
where
    U: crate::domain::ports::UserRepository + 'static,
    I: crate::domain::ports::InboundRepository + 'static,
{
    cfg.app_data(web::Data::new(state))
        .route("/traffic/all", web::get().to(get_all_traffic::<U, I>))
        .route("/traffic/{email}", web::get().to(get_traffic::<U, I>))
        .route(
            "/reset-traffic/{email}",
            web::post().to(reset_traffic::<U, I>),
        );
}
