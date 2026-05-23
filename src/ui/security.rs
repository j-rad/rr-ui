//! Security and Validation Logic
//!
//! Hardens the frontend by validating incoming gRPC telemetry and outgoing
//! configuration patches against strict schemas and whitelists.

use json_patch::Patch;
use serde_json::Value;

/// Validates a JSON Patch against a list of blocked paths to prevent
/// unauthorized modification of sensitive configuration fields.
pub fn validate_config_patch(patch: &Patch) -> Result<(), String> {
    // List of fields that are NEVER allowed to be patched via the UI
    let blocked_paths = [
        "/admin_port",
        "/private_key_path",
        "/log_path",
        "/db_path",
        "/core_binary_path",
    ];

    for op in patch.0.iter() {
        let path = match op {
            json_patch::PatchOperation::Add(op) => &op.path,
            json_patch::PatchOperation::Remove(op) => &op.path,
            json_patch::PatchOperation::Replace(op) => &op.path,
            json_patch::PatchOperation::Move(op) => &op.path,
            json_patch::PatchOperation::Copy(op) => &op.path,
            json_patch::PatchOperation::Test(op) => &op.path,
        };

        if blocked_paths.iter().any(|&blocked| path.starts_with(blocked)) {
            return Err(format!("Security violation: Modification of sensitive field '{}' is prohibited", path));
        }
    }

    Ok(())
}

/// Sanitizes input strings using a whitelist regex for SNI and IP fields.
pub fn sanitize_hostname(input: &str) -> bool {
    // Regex for valid hostname or IP
    let re = regex::Regex::new(r"^[a-zA-Z0-9\-\.]+$").unwrap();
    re.is_match(input)
}

/// Validates gRPC telemetry blobs against expected schema
pub fn validate_telemetry_blob(blob: &Value) -> bool {
    // Ensure basic structure exists
    if !blob.is_object() {
        return false;
    }
    
    // Check for essential telemetry fields
    let required_fields = ["cpu", "mem", "net_io"];
    for field in required_fields {
        if !blob.get(field).is_some() {
            return false;
        }
    }
    
    true
}
