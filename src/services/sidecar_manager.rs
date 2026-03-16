// src/services/sidecar_manager.rs
//! Sidecar Manager
//!
//! Manages plugin processes via Unix Domain Sockets

use crate::domain::plugin_api::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};

/// Resource limits for plugins
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_memory_mb: u64,
    pub max_cpu_percent: u8,
    pub max_disk_mb: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 100,
            max_cpu_percent: 10,
            max_disk_mb: 50,
        }
    }
}

/// Plugin process info
#[derive(Debug)]
struct PluginProcess {
    metadata: PluginMetadata,
    child: Child,
    socket_path: PathBuf,
    resource_limits: ResourceLimits,
    start_time: std::time::Instant,
    restart_count: u32,
}

/// Sidecar manager
pub struct SidecarManager {
    plugins: HashMap<String, PluginProcess>,
    plugin_dir: PathBuf,
    socket_dir: PathBuf,
    max_restart_attempts: u32,
}

impl SidecarManager {
    pub fn new(plugin_dir: PathBuf, socket_dir: PathBuf) -> Self {
        Self {
            plugins: HashMap::new(),
            plugin_dir,
            socket_dir,
            max_restart_attempts: 3,
        }
    }

    /// Load and start a plugin
    pub async fn load_plugin(&mut self, plugin_id: &str) -> Result<(), String> {
        let plugin_path = self.plugin_dir.join(plugin_id);
        let metadata_path = plugin_path.join("plugin.json");

        // Read metadata
        let metadata_content = std::fs::read_to_string(&metadata_path)
            .map_err(|e| format!("Failed to read plugin metadata: {}", e))?;

        let metadata: PluginMetadata = serde_json::from_str(&metadata_content)
            .map_err(|e| format!("Failed to parse plugin metadata: {}", e))?;

        // Create socket path
        let socket_path = self.socket_dir.join(format!("{}.sock", plugin_id));

        // Start plugin process
        let executable = plugin_path.join("main");
        let child = Command::new(&executable)
            .env("PLUGIN_SOCKET", &socket_path)
            .env("PLUGIN_ID", plugin_id)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn plugin process: {}", e))?;

        // Wait for socket to be created
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Store plugin info
        let process = PluginProcess {
            metadata: metadata.clone(),
            child,
            socket_path: socket_path.clone(),
            resource_limits: ResourceLimits::default(),
            start_time: std::time::Instant::now(),
            restart_count: 0,
        };

        self.plugins.insert(plugin_id.to_string(), process);

        // Send initialize message
        self.send_message(
            plugin_id,
            PluginMessage::Initialize {
                config: PluginConfig {
                    enabled: true,
                    settings: HashMap::new(),
                    ui_slots: Vec::new(),
                },
            },
        )
        .await?;

        Ok(())
    }

    /// Stop a plugin
    pub fn stop_plugin(&mut self, plugin_id: &str) -> Result<(), String> {
        if let Some(mut process) = self.plugins.remove(plugin_id) {
            process
                .child
                .kill()
                .map_err(|e| format!("Failed to kill plugin process: {}", e))?;

            // Clean up socket
            if process.socket_path.exists() {
                std::fs::remove_file(&process.socket_path)
                    .map_err(|e| format!("Failed to remove socket: {}", e))?;
            }

            Ok(())
        } else {
            Err(format!("Plugin {} not found", plugin_id))
        }
    }

    /// Send message to plugin
    pub async fn send_message(
        &self,
        plugin_id: &str,
        message: PluginMessage,
    ) -> Result<PluginResponse, String> {
        let process = self
            .plugins
            .get(plugin_id)
            .ok_or_else(|| format!("Plugin {} not found", plugin_id))?;

        let mut stream = UnixStream::connect(&process.socket_path)
            .await
            .map_err(|e| format!("Failed to connect to plugin socket: {}", e))?;

        // Send message
        let message_json = serde_json::to_string(&message)
            .map_err(|e| format!("Failed to serialize message: {}", e))?;

        stream
            .write_all(message_json.as_bytes())
            .await
            .map_err(|e| format!("Failed to write to socket: {}", e))?;

        stream
            .write_all(b"\n")
            .await
            .map_err(|e| format!("Failed to write newline: {}", e))?;

        // Read response
        let mut buffer = Vec::new();
        stream
            .read_to_end(&mut buffer)
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        let response: PluginResponse = serde_json::from_slice(&buffer)
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(response)
    }

    /// Invoke hook on all plugins that support it
    pub async fn invoke_hook(
        &self,
        hook: PluginHook,
    ) -> Vec<(String, Result<PluginResponse, String>)> {
        let mut results = Vec::new();

        for (plugin_id, process) in &self.plugins {
            // Check if plugin supports this hook type
            let supports_hook = match &hook {
                PluginHook::OnUserLimit { .. }
                | PluginHook::OnUserCreated { .. }
                | PluginHook::OnUserDeleted { .. } => process
                    .metadata
                    .capabilities
                    .contains(&PluginCapability::UserManagement),
                PluginHook::OnTrafficAlert { .. } => process
                    .metadata
                    .capabilities
                    .contains(&PluginCapability::TrafficMonitoring),
                _ => true,
            };

            if supports_hook {
                let result = self
                    .send_message(plugin_id, PluginMessage::InvokeHook { hook: hook.clone() })
                    .await;

                results.push((plugin_id.clone(), result));
            }
        }

        results
    }

    /// Monitor plugin health and restart if needed
    pub async fn monitor_plugins(&mut self) {
        let plugin_ids: Vec<String> = self.plugins.keys().cloned().collect();

        for plugin_id in plugin_ids {
            if let Some(process) = self.plugins.get_mut(&plugin_id) {
                // Check if process is still running
                match process.child.try_wait() {
                    Ok(Some(status)) => {
                        log::warn!("Plugin {} exited with status: {:?}", plugin_id, status);

                        // Attempt restart if under limit
                        if process.restart_count < self.max_restart_attempts {
                            log::info!(
                                "Restarting plugin {} (attempt {})",
                                plugin_id,
                                process.restart_count + 1
                            );

                            let plugin_id_clone = plugin_id.clone();
                            drop(process); // Release borrow

                            self.stop_plugin(&plugin_id_clone).ok();
                            if let Err(e) = self.load_plugin(&plugin_id_clone).await {
                                log::error!("Failed to restart plugin {}: {}", plugin_id_clone, e);
                            } else {
                                if let Some(p) = self.plugins.get_mut(&plugin_id_clone) {
                                    p.restart_count += 1;
                                }
                            }
                        } else {
                            log::error!("Plugin {} exceeded max restart attempts", plugin_id);
                        }
                    }
                    Ok(None) => {
                        // Still running - send heartbeat
                        if let Err(e) = self
                            .send_message(&plugin_id, PluginMessage::Heartbeat)
                            .await
                        {
                            log::warn!("Plugin {} heartbeat failed: {}", plugin_id, e);
                        }
                    }
                    Err(e) => {
                        log::error!("Error checking plugin {} status: {}", plugin_id, e);
                    }
                }
            }
        }
    }

    /// Get all UI slots from plugins
    pub async fn get_ui_slots(&self) -> HashMap<String, Vec<UiSlot>> {
        let mut slots = HashMap::new();

        for (plugin_id, _) in &self.plugins {
            if let Ok(response) = self
                .send_message(plugin_id, PluginMessage::GetUiSlots)
                .await
            {
                if let Some(data) = response.data {
                    if let Ok(plugin_slots) = serde_json::from_value::<Vec<UiSlot>>(data) {
                        slots.insert(plugin_id.clone(), plugin_slots);
                    }
                }
            }
        }

        slots
    }

    /// Get plugin status
    pub fn get_plugin_status(&mut self, plugin_id: &str) -> Option<PluginStatus> {
        self.plugins
            .get_mut(plugin_id)
            .map(|process| match process.child.try_wait() {
                Ok(Some(_)) => PluginStatus::Failed,
                Ok(None) => PluginStatus::Running,
                Err(_) => PluginStatus::Failed,
            })
    }

    /// List all loaded plugins
    pub fn list_plugins(&self) -> Vec<PluginMetadata> {
        self.plugins.values().map(|p| p.metadata.clone()).collect()
    }
}

impl Drop for SidecarManager {
    fn drop(&mut self) {
        // Clean shutdown of all plugins
        let plugin_ids: Vec<String> = self.plugins.keys().cloned().collect();
        for plugin_id in plugin_ids {
            self.stop_plugin(&plugin_id).ok();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.max_memory_mb, 100);
        assert_eq!(limits.max_cpu_percent, 10);
    }

    #[test]
    fn test_sidecar_manager_creation() {
        let manager =
            SidecarManager::new(PathBuf::from("/tmp/plugins"), PathBuf::from("/tmp/sockets"));

        assert_eq!(manager.plugins.len(), 0);
        assert_eq!(manager.max_restart_attempts, 3);
    }
}
