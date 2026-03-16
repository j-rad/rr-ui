// src/domain/bulk_operations.rs
//! Bulk Operations Domain Types
//!
//! Defines types for mass client management operations

use serde::{Deserialize, Serialize};

/// Types of bulk operations that can be performed on clients
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BulkOperation {
    /// Add days to client expiration
    ExtendExpiry { days: i64 },
    /// Set absolute expiry date (Unix timestamp)
    SetExpiry { timestamp: i64 },
    /// Set traffic quota in GB
    SetQuota { total_gb: u64 },
    /// Add traffic to existing quota
    AddQuota { additional_gb: u64 },
    /// Reset traffic counters to zero
    ResetTraffic,
    /// Enable selected clients
    Enable,
    /// Disable selected clients
    Disable,
    /// Delete selected clients
    Delete,
    /// Update speed limits
    SetSpeedLimit {
        download_mbps: Option<u32>,
        upload_mbps: Option<u32>,
    },
    /// Move to different inbound
    MoveToInbound { inbound_id: String },
}

impl BulkOperation {
    pub fn display_name(&self) -> &'static str {
        match self {
            BulkOperation::ExtendExpiry { .. } => "Extend Expiry",
            BulkOperation::SetExpiry { .. } => "Set Expiry Date",
            BulkOperation::SetQuota { .. } => "Set Traffic Quota",
            BulkOperation::AddQuota { .. } => "Add Traffic Quota",
            BulkOperation::ResetTraffic => "Reset Traffic",
            BulkOperation::Enable => "Enable Clients",
            BulkOperation::Disable => "Disable Clients",
            BulkOperation::Delete => "Delete Clients",
            BulkOperation::SetSpeedLimit { .. } => "Set Speed Limit",
            BulkOperation::MoveToInbound { .. } => "Move to Inbound",
        }
    }

    pub fn is_destructive(&self) -> bool {
        matches!(self, BulkOperation::Delete | BulkOperation::ResetTraffic)
    }
}

/// Request for bulk operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkOperationRequest {
    /// IDs of clients to operate on
    pub client_ids: Vec<String>,
    /// Operation to perform
    pub operation: BulkOperation,
    /// Optional: Only perform on clients matching filter
    pub filter: Option<ClientFilter>,
}

/// Filter for client selection
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClientFilter {
    pub inbound_id: Option<String>,
    pub enabled: Option<bool>,
    pub expired: Option<bool>,
    pub over_quota: Option<bool>,
}

/// Progress of a bulk operation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BulkOperationProgress {
    /// Total number of clients to process
    pub total: usize,
    /// Number of clients processed so far
    pub processed: usize,
    /// Number of successful operations
    pub succeeded: usize,
    /// Number of failed operations
    pub failed: usize,
    /// Error messages for failed operations
    pub errors: Vec<BulkOperationError>,
    /// Whether operation is complete
    pub is_complete: bool,
    /// Whether operation was cancelled
    pub is_cancelled: bool,
}

impl BulkOperationProgress {
    pub fn new(total: usize) -> Self {
        Self {
            total,
            processed: 0,
            succeeded: 0,
            failed: 0,
            errors: Vec::new(),
            is_complete: false,
            is_cancelled: false,
        }
    }

    pub fn record_success(&mut self) {
        self.processed += 1;
        self.succeeded += 1;
    }

    pub fn record_failure(&mut self, client_id: String, error: String) {
        self.processed += 1;
        self.failed += 1;
        self.errors.push(BulkOperationError { client_id, error });
    }

    pub fn mark_complete(&mut self) {
        self.is_complete = true;
    }

    pub fn mark_cancelled(&mut self) {
        self.is_cancelled = true;
        self.is_complete = true;
    }

    pub fn percent_complete(&self) -> f64 {
        if self.total == 0 {
            100.0
        } else {
            (self.processed as f64 / self.total as f64) * 100.0
        }
    }
}

/// Error for a single client operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkOperationError {
    pub client_id: String,
    pub error: String,
}

/// Result of a bulk operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkOperationResult {
    /// Operation ID for tracking
    pub operation_id: String,
    /// Final progress state
    pub progress: BulkOperationProgress,
    /// Time taken in milliseconds
    pub duration_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_tracking() {
        let mut progress = BulkOperationProgress::new(10);
        assert_eq!(progress.percent_complete(), 0.0);

        progress.record_success();
        progress.record_success();
        assert_eq!(progress.processed, 2);
        assert_eq!(progress.percent_complete(), 20.0);

        progress.record_failure("client-1".into(), "Error".into());
        assert_eq!(progress.failed, 1);
        assert_eq!(progress.errors.len(), 1);
    }

    #[test]
    fn test_operation_display_name() {
        assert_eq!(BulkOperation::Enable.display_name(), "Enable Clients");
        assert_eq!(BulkOperation::Delete.display_name(), "Delete Clients");
    }

    #[test]
    fn test_is_destructive() {
        assert!(BulkOperation::Delete.is_destructive());
        assert!(BulkOperation::ResetTraffic.is_destructive());
        assert!(!BulkOperation::Enable.is_destructive());
    }
}
