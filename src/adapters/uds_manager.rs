// src/adapters/uds_manager.rs
use crate::db::DbClient;
use crate::models::AllSetting;
use crate::repositories::setting::SettingOps;
use crate::services::auth::hash_password;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use sysinfo::{CpuExt, System, SystemExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};

const SOCKET_PATH: &str = "/run/rr-ui/rr-ui.sock";

#[derive(Debug, Serialize, Deserialize)]
pub enum UdsRequest {
    GetStatus,
    GetSettings,
    ResetPassword { new_password: String },
    GetLogs { lines: usize },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStatus {
    pub uptime_seconds: u64,
    pub memory_mb: u64,
    pub cpu_percent: f32,
    pub active_connections: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PanelSettings {
    pub port: u16,
    pub username: String,
    pub db_path: String,
    pub two_factor_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UdsResponse {
    Status(SystemStatus),
    Settings(PanelSettings),
    PasswordReset { success: bool, message: String },
    Logs(Vec<String>),
    Error(String),
}

pub struct UdsManager {
    socket_path: PathBuf,
    db_client: DbClient,
    system: Arc<Mutex<System>>,
}

impl UdsManager {
    pub fn new(db_client: DbClient) -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self {
            socket_path: PathBuf::from(SOCKET_PATH),
            db_client,
            system: Arc::new(Mutex::new(sys)),
        }
    }

    pub async fn start_listener(self: Arc<Self>) -> Result<()> {
        // Remove old socket if exists
        let _ = std::fs::remove_file(&self.socket_path);

        if let Some(parent) = self.socket_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let listener = UnixListener::bind(&self.socket_path)?;
        log::info!("UDS listener started at {:?}", self.socket_path);

        // Set permissions to allow CLI access
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(&self.socket_path) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o666);
                let _ = std::fs::set_permissions(&self.socket_path, perms);
            }
        }

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let manager = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = manager.handle_connection(stream).await {
                            log::error!("UDS connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    log::error!("UDS accept error: {}", e);
                }
            }
        }
    }

    async fn handle_connection(&self, mut stream: UnixStream) -> Result<()> {
        // Read request length (4 bytes)
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await?;
        let len = u32::from_le_bytes(len_buf) as usize;

        // Read request data
        let mut data = vec![0u8; len];
        stream.read_exact(&mut data).await?;

        let request: UdsRequest = bincode::deserialize(&data)?;
        let response = self.process_request(request).await;

        // Send response
        let response_data = bincode::serialize(&response)?;
        let response_len = (response_data.len() as u32).to_le_bytes();
        stream.write_all(&response_len).await?;
        stream.write_all(&response_data).await?;

        Ok(())
    }

    async fn process_request(&self, request: UdsRequest) -> UdsResponse {
        match request {
            UdsRequest::GetStatus => {
                // Get system metrics
                let mut sys = self.system.lock().unwrap();
                sys.refresh_all();

                let uptime = sys.uptime();
                let memory = sys.used_memory() / 1024 / 1024; // Convert to MB
                let cpu = sys.global_cpu_info().cpu_usage();

                // For now, we return 0 or placeholder as full integration requires RustRay client
                UdsResponse::Status(SystemStatus {
                    uptime_seconds: uptime,
                    memory_mb: memory,
                    cpu_percent: cpu,
                    active_connections: 0,
                })
            }
            UdsRequest::GetSettings => {
                match <AllSetting as SettingOps>::get(&self.db_client).await {
                    Ok(Some(settings)) => UdsResponse::Settings(PanelSettings {
                        port: settings.web_port,
                        username: settings.username,
                        db_path: "rr-ui.db".to_string(), // In-memory or file path from env
                        two_factor_enabled: settings.is_two_factor_enabled,
                    }),
                    Ok(None) => UdsResponse::Error("Settings not initialized".to_string()),
                    Err(e) => UdsResponse::Error(format!("DB Error: {}", e)),
                }
            }
            UdsRequest::ResetPassword { new_password } => {
                match <AllSetting as SettingOps>::get(&self.db_client).await {
                    Ok(Some(mut settings)) => match hash_password(&new_password) {
                        Ok(hash) => {
                            settings.password_hash = hash;
                            match settings.save(&self.db_client).await {
                                Ok(_) => {
                                    log::info!("Password reset successfully via UDS");
                                    UdsResponse::PasswordReset {
                                        success: true,
                                        message: "Password updated successfully".to_string(),
                                    }
                                }
                                Err(e) => {
                                    UdsResponse::Error(format!("Failed to save settings: {}", e))
                                }
                            }
                        }
                        Err(e) => UdsResponse::Error(format!("Failed to hash password: {}", e)),
                    },
                    Ok(None) => UdsResponse::Error("Settings not found".to_string()),
                    Err(e) => UdsResponse::Error(format!("DB Error: {}", e)),
                }
            }
            UdsRequest::GetLogs { lines } => {
                let logs = Self::read_logs(lines).unwrap_or_default();
                UdsResponse::Logs(logs)
            }
        }
    }

    fn read_logs(lines: usize) -> Result<Vec<String>> {
        use std::process::Command;

        let output = Command::new("journalctl")
            .args(&["-u", "rr-ui", "-n", &lines.to_string(), "--no-pager"])
            .output()?;

        let logs = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect();

        Ok(logs)
    }
}

// Client-side helper for CLI
pub struct UdsClient {
    socket_path: PathBuf,
}

impl UdsClient {
    pub fn new() -> Self {
        Self {
            socket_path: PathBuf::from(SOCKET_PATH),
        }
    }

    pub async fn send_request(&self, request: UdsRequest) -> Result<UdsResponse> {
        let mut stream = UnixStream::connect(&self.socket_path).await?;

        // Send request
        let data = bincode::serialize(&request)?;
        let len = (data.len() as u32).to_le_bytes();
        stream.write_all(&len).await?;
        stream.write_all(&data).await?;

        // Read response length
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await?;
        let len = u32::from_le_bytes(len_buf) as usize;

        // Read response data
        let mut response_data = vec![0u8; len];
        stream.read_exact(&mut response_data).await?;

        let response: UdsResponse = bincode::deserialize(&response_data)?;
        Ok(response)
    }
}
