// src/domain/plugin_api.rs
//! Plugin API Definitions
//!
//! Defines the plugin interface and lifecycle hooks

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub capabilities: Vec<PluginCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PluginCapability {
    UserManagement,
    TrafficMonitoring,
    CustomProtocol,
    UiExtension,
    AlertHandler,
    ConfigValidator,
}

/// Plugin lifecycle hooks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "hook", rename_all = "snake_case")]
pub enum PluginHook {
    OnUserLimit {
        user_id: String,
        quota_gb: u64,
        used_gb: f64,
    },
    OnNodeCensored {
        node_id: String,
        reason: String,
        timestamp: i64,
    },
    OnConfigChange {
        config_type: String,
        old_value: serde_json::Value,
        new_value: serde_json::Value,
    },
    OnTrafficAlert {
        alert_type: String,
        threshold: f64,
        current: f64,
    },
    OnUserCreated {
        user_id: String,
        email: String,
    },
    OnUserDeleted {
        user_id: String,
    },
}

/// Plugin response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResponse {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<serde_json::Value>,
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled: bool,
    pub settings: HashMap<String, serde_json::Value>,
    pub ui_slots: Vec<UiSlot>,
}

/// UI slot definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UiSlot {
    pub slot_id: String,
    pub position: UiPosition,
    pub component_url: String,
    pub props: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum UiPosition {
    DashboardTop,
    DashboardBottom,
    SidebarTop,
    SidebarBottom,
    SettingsTab,
    UserDetailsPanel,
}

/// Plugin registry entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRegistryEntry {
    pub metadata: PluginMetadata,
    pub config: PluginConfig,
    pub socket_path: String,
    pub status: PluginStatus,
    pub last_heartbeat: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PluginStatus {
    Running,
    Stopped,
    Failed,
    Disabled,
}

/// Plugin message protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PluginMessage {
    Initialize {
        config: PluginConfig,
    },
    Shutdown,
    Heartbeat,
    InvokeHook {
        hook: PluginHook,
    },
    GetUiSlots,
    Response {
        request_id: String,
        response: PluginResponse,
    },
}

/// Plugin development guide
pub const PLUGIN_DEV_GUIDE: &str = r#"
# Plugin Development Guide

## Overview
Plugins extend rr-ui functionality through a Unix Domain Socket (UDS) interface.

## Plugin Structure
```
my-plugin/
├── plugin.json          # Metadata
├── main                 # Executable
└── ui/
    └── component.js     # Optional UI component
```

## plugin.json Example
```json
{
  "id": "com.example.myplugin",
  "name": "My Plugin",
  "version": "1.0.0",
  "author": "Your Name",
  "description": "Does something cool",
  "capabilities": ["user_management", "ui_extension"]
}
```

## Communication Protocol
Plugins communicate via JSON messages over UDS:

1. **Initialize**: Receive configuration
2. **Heartbeat**: Respond every 30s
3. **InvokeHook**: Handle lifecycle events
4. **Response**: Return results

## Example Plugin (Rust)
```rust
use tokio::net::UnixListener;

#[tokio::main]
async fn main() {
    let listener = UnixListener::bind("/tmp/myplugin.sock").unwrap();
    
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        tokio::spawn(handle_connection(stream));
    }
}

async fn handle_connection(stream: UnixStream) {
    // Read PluginMessage
    // Process hook
    // Send PluginResponse
}
```

## Hooks
- `on_user_limit`: User quota exceeded
- `on_node_censored`: Node blocked
- `on_config_change`: Configuration updated
- `on_traffic_alert`: Traffic threshold exceeded

## UI Slots
Inject custom UI components:
```json
{
  "slot_id": "my_dashboard_widget",
  "position": "dashboard_top",
  "component_url": "/plugins/myplugin/widget.js",
  "props": {
    "title": "My Widget"
  }
}
```

## Resource Limits
- Memory: 100MB
- CPU: 10%
- Disk: 50MB
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_metadata_serialization() {
        let metadata = PluginMetadata {
            id: "test.plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            author: "Test Author".to_string(),
            description: "A test plugin".to_string(),
            capabilities: vec![PluginCapability::UserManagement],
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: PluginMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(metadata.id, deserialized.id);
    }

    #[test]
    fn test_plugin_hook_serialization() {
        let hook = PluginHook::OnUserLimit {
            user_id: "user123".to_string(),
            quota_gb: 100,
            used_gb: 95.5,
        };

        let json = serde_json::to_string(&hook).unwrap();
        assert!(json.contains("on_user_limit"));
    }
}
