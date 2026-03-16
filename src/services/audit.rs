// src/services/audit.rs
//! Audit Service for Security Logging
//!
//! Records security-related events to the database for compliance and forensics.

use crate::db::DbClient;
use crate::models::{AuditAction, AuditEvent};
use log::{error, info};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Audit service for recording security events
#[derive(Clone)]
pub struct AuditService {
    db: Arc<RwLock<DbClient>>,
}

impl AuditService {
    /// Create a new audit service with a database connection
    pub fn new(db: Arc<RwLock<DbClient>>) -> Self {
        Self { db }
    }

    /// Record an audit event to the database
    pub async fn log(&self, event: AuditEvent) -> Result<(), String> {
        let db = self.db.read().await;

        match db
            .client
            .create::<Option<AuditEvent>>("audit_log")
            .content(event.clone())
            .await
        {
            Ok(_) => {
                info!(
                    "Audit: {:?} by {} from {}",
                    event.action,
                    event.user.as_deref().unwrap_or("anonymous"),
                    event.ip_address.as_deref().unwrap_or("unknown")
                );
                Ok(())
            }
            Err(e) => {
                error!("Failed to record audit event: {}", e);
                Err(format!("Failed to record audit event: {}", e))
            }
        }
    }

    /// Record a successful login
    pub async fn log_login(&self, user: &str, ip: &str, user_agent: Option<&str>) {
        let mut event = AuditEvent::new(AuditAction::Login)
            .with_user(user)
            .with_ip(ip);

        if let Some(ua) = user_agent {
            event = event.with_user_agent(ua);
        }

        let _ = self.log(event).await;
    }

    /// Record a failed login attempt
    pub async fn log_login_failed(&self, user: &str, ip: &str, reason: &str) {
        let event = AuditEvent::failed(AuditAction::LoginFailed, reason)
            .with_user(user)
            .with_ip(ip);

        let _ = self.log(event).await;
    }

    /// Record an inbound creation
    pub async fn log_inbound_created(&self, user: &str, inbound_tag: &str) {
        let event = AuditEvent::new(AuditAction::InboundCreated)
            .with_user(user)
            .with_details(serde_json::json!({ "tag": inbound_tag }));

        let _ = self.log(event).await;
    }

    /// Record a client creation
    pub async fn log_client_created(&self, user: &str, client_email: &str, inbound_tag: &str) {
        let event = AuditEvent::new(AuditAction::ClientCreated)
            .with_user(user)
            .with_details(serde_json::json!({
                "email": client_email,
                "inbound": inbound_tag
            }));

        let _ = self.log(event).await;
    }

    /// Record a settings update
    pub async fn log_settings_updated(&self, user: &str, changes: serde_json::Value) {
        let event = AuditEvent::new(AuditAction::SettingsUpdated)
            .with_user(user)
            .with_details(changes);

        let _ = self.log(event).await;
    }

    /// Record an IP ban
    pub async fn log_ip_banned(&self, ip: &str, reason: &str) {
        let event = AuditEvent::new(AuditAction::IpBanned)
            .with_ip(ip)
            .with_details(serde_json::json!({ "reason": reason }));

        let _ = self.log(event).await;
    }

    /// Record core start
    pub async fn log_core_started(&self, user: Option<&str>) {
        let mut event = AuditEvent::new(AuditAction::CoreStarted);
        if let Some(u) = user {
            event = event.with_user(u);
        }
        let _ = self.log(event).await;
    }

    /// Record core stop
    pub async fn log_core_stopped(&self, user: Option<&str>) {
        let mut event = AuditEvent::new(AuditAction::CoreStopped);
        if let Some(u) = user {
            event = event.with_user(u);
        }
        let _ = self.log(event).await;
    }

    /// Query recent audit events
    pub async fn get_recent(&self, limit: usize) -> Result<Vec<AuditEvent>, String> {
        let db = self.db.read().await;

        let query = format!(
            "SELECT * FROM audit_log ORDER BY timestamp DESC LIMIT {}",
            limit
        );

        match db.client.query(&query).await {
            Ok(mut response) => {
                let events: Vec<AuditEvent> = response.take(0).unwrap_or_default();
                Ok(events)
            }
            Err(e) => Err(format!("Failed to query audit log: {}", e)),
        }
    }

    /// Query audit events by action type
    pub async fn get_by_action(
        &self,
        action: AuditAction,
        limit: usize,
    ) -> Result<Vec<AuditEvent>, String> {
        let db = self.db.read().await;
        let action_str = serde_json::to_string(&action)
            .unwrap_or_default()
            .trim_matches('"')
            .to_string();

        let query = format!(
            "SELECT * FROM audit_log WHERE action = '{}' ORDER BY timestamp DESC LIMIT {}",
            action_str, limit
        );

        match db.client.query(&query).await {
            Ok(mut response) => {
                let events: Vec<AuditEvent> = response.take(0).unwrap_or_default();
                Ok(events)
            }
            Err(e) => Err(format!("Failed to query audit log: {}", e)),
        }
    }

    /// Query audit events by user
    pub async fn get_by_user(&self, user: &str, limit: usize) -> Result<Vec<AuditEvent>, String> {
        let db = self.db.read().await;

        let query = format!(
            "SELECT * FROM audit_log WHERE user = '{}' ORDER BY timestamp DESC LIMIT {}",
            user, limit
        );

        match db.client.query(&query).await {
            Ok(mut response) => {
                let events: Vec<AuditEvent> = response.take(0).unwrap_or_default();
                Ok(events)
            }
            Err(e) => Err(format!("Failed to query audit log: {}", e)),
        }
    }
}

/// Shared audit service type for Actix web data
pub type SharedAuditService = Arc<AuditService>;
