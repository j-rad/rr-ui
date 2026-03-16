// src/rustray_process.rs
use crate::db::DbClient;
use crate::models::AllSetting;
use crate::repositories::setting::SettingOps;
use crate::rustray_config::{RustRayConfig, RustRayConfigBuilder};
use log::{error, info, warn};
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::Arc;

pub struct RustRayProcess {
    child: Option<Child>,
    bin_path: PathBuf,
    config_path: PathBuf,
    api_port: Option<u16>,
}

impl RustRayProcess {
    pub fn new(bin_path: &str, config_path: &str, api_port: Option<u16>) -> Self {
        Self {
            child: None,
            bin_path: PathBuf::from(bin_path),
            config_path: PathBuf::from(config_path),
            api_port,
        }
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        if self.is_running() {
            warn!("rustray is already running.");
            return Ok(());
        }

        // Validate binary exists
        if !self.bin_path.exists() {
            let err_msg = format!(
                "Binary not found at {:?}. Please check your core settings or ensure rustray is in PATH.",
                self.bin_path
            );
            error!("{}", err_msg);
            return Err(anyhow::anyhow!(err_msg));
        }

        info!("Starting rustray core at {:?}...", self.bin_path);

        let mut cmd = Command::new(&self.bin_path);

        // RustRay invocation: rustray --config config.json
        cmd.arg("--config").arg(&self.config_path);

        match cmd.spawn() {
            Ok(child) => {
                self.child = Some(child);
                info!("rustray core started successfully.");
                Ok(())
            }
            Err(e) => {
                let err_msg = format!("Failed to start rustray core: {}", e);
                error!("{}", err_msg);
                Err(anyhow::anyhow!(err_msg))
            }
        }
    }

    pub fn stop(&mut self) -> anyhow::Result<()> {
        if let Some(mut child) = self.child.take() {
            info!("Stopping rustray core...");
            child.kill()?;
            child.wait()?; // Prevent zombies
            info!("rustray core stopped.");
        }
        Ok(())
    }

    pub async fn restart(&mut self, db: &DbClient) -> anyhow::Result<()> {
        info!("Restarting core process...");

        // Fetch new settings first
        let settings: Result<Option<AllSetting>, _> = db.client.select(("setting", "global")).await;

        if let Ok(Some(s)) = settings {
            info!("Updating core settings: path={:?}", s.core_path);

            // Update binary path
            self.bin_path = PathBuf::from(s.core_path.unwrap_or_else(|| {
                let candidates = [
                    "/usr/local/rr-ui/bin/rustray",
                    "./rustray/target/release/rustray",
                    "/usr/local/bin/rustray",
                    "rustray",
                ];
                for cand in candidates {
                    if std::path::Path::new(cand).exists() {
                        return cand.to_string();
                    }
                }
                "rustray".to_string()
            }));
        } else {
            warn!("No settings found in DB during restart, keeping current configuration");
        }

        self.stop()?;

        // Regenerate config
        match RustRayConfigBuilder::build(db).await {
            Ok(mut config) => {
                // Inject Dynamic Port if set
                if let Some(port) = self.api_port {
                    if let Some(inbound) = config.inbounds.iter_mut().find(|i| i.tag == "api") {
                        inbound.port = port as u32;
                        info!("Injected dynamic API port {} into config", port);
                    }
                }

                // Use Atomic Persistence Layer for crash-safe writes
                use crate::adapters::atomic_config::AtomicConfigWriter;

                let writer = AtomicConfigWriter::new(
                    self.config_path
                        .parent()
                        .unwrap_or_else(|| std::path::Path::new(".")),
                );

                writer.write_json(&self.config_path, &config)?;
                info!("RustRay config updated atomically with backup.");
            }
            Err(e) => {
                error!("Failed to generate RustRay config: {}", e);
                return Err(e);
            }
        }

        self.start()?;
        Ok(())
    }

    pub fn is_running(&mut self) -> bool {
        if let Some(child) = &mut self.child {
            match child.try_wait() {
                Ok(Some(_)) => false, // Exited
                Ok(None) => true,     // Running
                Err(_) => false,
            }
        } else {
            false
        }
    }
}

impl Drop for RustRayProcess {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            // Check if it's still running
            let is_running = match child.try_wait() {
                Ok(None) => true,
                _ => false,
            };

            if is_running {
                warn!("RustRayProcess dropped but child is still running. Killing it...");
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }
}

// Shared State Wrapper for Actix
#[derive(Clone)]
pub struct SharedRustRayProcess {
    pub process: Arc<tokio::sync::Mutex<RustRayProcess>>,
}

impl SharedRustRayProcess {
    pub fn new(bin_path: &str, config_path: &str) -> Self {
        Self {
            process: Arc::new(tokio::sync::Mutex::new(RustRayProcess::new(
                bin_path,
                config_path,
                None,
            ))),
        }
    }

    /// Initialize the process with settings from database
    pub async fn init_from_db(config_path: &str, db: &DbClient) -> Self {
        Self::init_from_db_with_port(config_path, db, 10085).await // Fallback default
    }

    pub async fn init_from_db_with_port(config_path: &str, db: &DbClient, api_port: u16) -> Self {
        let settings: Result<Option<AllSetting>, _> = db.client.select(("setting", "global")).await;

        let bin_path = if let Ok(Some(s)) = settings {
            let path = s.core_path.unwrap_or_else(|| {
                // Try several common locations before falling back to PATH
                let candidates = [
                    "/usr/local/rr-ui/bin/rustray",
                    "./rustray/target/release/rustray",
                    "/usr/local/bin/rustray",
                    "rustray",
                ];

                for cand in candidates {
                    if std::path::Path::new(cand).exists() {
                        return cand.to_string();
                    }
                }

                "rustray".to_string()
            });
            path
        } else {
            // Fallback to defaults if no settings found
            info!("No settings found in DB, attempting to locate core: rustray");
            let candidates = [
                "/usr/local/rr-ui/bin/rustray",
                "./rustray/target/release/rustray",
                "/usr/local/bin/rustray",
                "rustray",
            ];

            let path = candidates
                .iter()
                .find(|&&c| std::path::Path::new(c).exists())
                .unwrap_or(&"rustray")
                .to_string();

            path
        };

        let mut process = RustRayProcess::new(&bin_path, config_path, Some(api_port));

        // Auto-start or at least generate config?
        // Usually restarting triggers config gen and start.
        if let Err(e) = process.restart(db).await {
            error!("Failed to start RustRay process during init: {}", e);
        }

        Self {
            process: Arc::new(tokio::sync::Mutex::new(process)),
        }
    }
}

/// Resolves the RustRay binary path from database settings.
///
/// # Arguments
///
/// * `db` - A reference to the `DbClient`.
///
/// # Returns
///
/// A `Result` containing a binary_path.
/// Returns an error if the binary is not found at the resolved path.
pub async fn get_rustray_binary_path(db: &DbClient) -> anyhow::Result<PathBuf> {
    let settings = <AllSetting as SettingOps>::get(db).await?;
    let bin_path_str = if let Some(s) = settings {
        let path = s.core_path.unwrap_or_else(|| {
            let candidates = [
                "/usr/local/rr-ui/bin/rustray",
                "./rustray/target/release/rustray",
                "/usr/local/bin/rustray",
                "rustray",
            ];
            for cand in candidates {
                if std::path::Path::new(cand).exists() {
                    return cand.to_string();
                }
            }
            "rustray".to_string()
        });
        path
    } else {
        let candidates = [
            "/usr/local/rr-ui/bin/rustray",
            "./rustray/target/release/rustray",
            "/usr/local/bin/rustray",
            "rustray",
        ];
        let path = candidates
            .iter()
            .find(|&&c| std::path::Path::new(c).exists())
            .unwrap_or(&"rustray")
            .to_string();
        path
    };

    let bin_path = PathBuf::from(&bin_path_str);

    // Check if binary exists
    if !bin_path.exists() {
        // Try to find in PATH if it's just a binary name
        if !bin_path_str.contains('/') && !bin_path_str.contains('\\') {
            // It's just a binary name, might be in PATH
            // We'll let the command execution handle this
            return Ok(bin_path);
        }
        return Err(anyhow::anyhow!(
            "RustRay binary not found at {}",
            bin_path.display()
        ));
    }

    Ok(bin_path)
}

/// Validates the provided configuration by running a dry-run with the core binary.
/// (RustRay doesn't support dry run easily, but leaving placeholder)
pub async fn validate_config(config: &RustRayConfig<'_>, db: &DbClient) -> anyhow::Result<()> {
    info!("Skipping config validation for RustRay core (dry-run not supported).");
    Ok(())
}
