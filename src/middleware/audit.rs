// src/middleware/audit.rs
//! Middleware for logging audit events from API requests

use crate::models::{AuditAction, AuditEvent};
use crate::services::audit::SharedAuditService;
use actix_web::{
    Error, HttpMessage,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
};
use futures_util::future::{LocalBoxFuture, Ready, ok};
use std::rc::Rc;

/// Audit Middleware for logging API access
pub struct AuditLog {
    audit_service: SharedAuditService,
}

impl AuditLog {
    pub fn new(audit_service: SharedAuditService) -> Self {
        Self { audit_service }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuditLog
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuditLogMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuditLogMiddleware {
            service: Rc::new(service),
            audit_service: self.audit_service.clone(),
        })
    }
}

pub struct AuditLogMiddleware<S> {
    service: Rc<S>,
    audit_service: SharedAuditService,
}

impl<S, B> Service<ServiceRequest> for AuditLogMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        ctx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let audit_service = self.audit_service.clone();
        let path = req.path().to_string();
        let method = req.method().to_string();

        let should_log_access = !path.starts_with("/static") && !path.starts_with("/favicon.ico");

        // Extract IP and User from request (if Auth middleware ran before)
        let ip = req.peer_addr().map(|a| a.ip().to_string());

        // Clone user from extensions if available (populated by Auth middleware)
        let user = if let Some(claims) = req.extensions().get::<crate::services::auth::Claims>() {
            Some(claims.sub.clone())
        } else {
            None
        };

        // Proceed with request
        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;

            // Only log if successful response or specific error codes, and if path is relevant
            if should_log_access {
                let status = res.status();

                // Construct action based on method/path heuristic
                let action = if path.contains("/login") {
                    if status.is_success() {
                        AuditAction::Login
                    } else {
                        AuditAction::LoginFailed
                    }
                } else if path.contains("/inbounds") && method == "POST" {
                    AuditAction::InboundCreated
                } else if path.contains("/inbounds") && method == "DELETE" {
                    AuditAction::InboundDeleted
                } else if path.contains("/settings") && method == "POST" {
                    AuditAction::SettingsUpdated
                } else {
                    // Generic access log doesn't map to specific AuditAction, maybe skip or map to Unknown?
                    // For now, let's only log high-value targets or if it's a mutation (POST/PUT/DELETE)
                    if method != "GET" || path.contains("/login") {
                        AuditAction::Unknown
                    } else {
                        // Skip GET requests to avoid noise unless configured
                        return Ok(res);
                    }
                };

                let mut event = AuditEvent::new(action).with_details(serde_json::json!({
                    "method": method,
                    "path": path,
                    "status": status.as_u16()
                }));

                if let Some(ip_addr) = ip {
                    event = event.with_ip(ip_addr);
                }

                if let Some(username) = user {
                    event = event.with_user(username);
                }

                if !status.is_success() {
                    event = AuditEvent::failed(event.action, format!("HTTP {}", status));
                }

                // Fire and forget logging
                let _ = audit_service.log(event).await;
            }

            Ok(res)
        })
    }
}
