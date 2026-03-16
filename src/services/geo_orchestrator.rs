use anyhow::{anyhow, Context, Result};
use log::info;
use reqwest::header::CONTENT_LENGTH;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoAssetInfo {
    pub version: String,
    pub file_size: u64,
    pub last_update: chrono::DateTime<chrono::Utc>,
    pub checksum: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GeoOrchestratorState {
    pub geoip_info: Option<GeoAssetInfo>,
    pub geosite_info: Option<GeoAssetInfo>,
    pub is_syncing: bool,
    pub last_sync_result: Option<Result<(), String>>,
}

pub struct GeoOrchestrator {
    state: Arc<RwLock<GeoOrchestratorState>>,
    http_client: Client,
    data_dir: PathBuf,
}

impl GeoOrchestrator {
    pub fn new(data_dir: impl AsRef<Path>) -> Result<Self> {
        let path = data_dir.as_ref().to_path_buf();
        if !path.exists() {
            std::fs::create_dir_all(&path)?;
        }

        Ok(Self {
            state: Arc::new(RwLock::new(GeoOrchestratorState::default())),
            http_client: Client::builder()
                .user_agent("rr-ui-geo-orchestrator/1.0")
                .build()?,
            data_dir: path,
        })
    }

    pub async fn initialize(&self) -> Result<()> {
        let mut state = self.state.write().await;

        // Load existing state if available
        if let Ok(info) = self.load_asset_info("geoip.dat").await {
            state.geoip_info = Some(info);
        }
        if let Ok(info) = self.load_asset_info("geosite.dat").await {
            state.geosite_info = Some(info);
        }

        Ok(())
    }

    pub async fn get_state(&self) -> GeoOrchestratorState {
        self.state.read().await.clone()
    }

    async fn load_asset_info(&self, filename: &str) -> Result<GeoAssetInfo> {
        let path = self.data_dir.join(filename);
        let metadata = fs::metadata(&path).await?;
        let chksum = self.calculate_file_checksum(&path).await?;

        Ok(GeoAssetInfo {
            version: "local".to_string(), // Metadata doesn't store version, assume local
            file_size: metadata.len(),
            last_update: chrono::Utc::now(),
            checksum: chksum,
        })
    }

    async fn calculate_file_checksum(&self, path: &Path) -> Result<String> {
        let mut file = File::open(path).await?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];

        use tokio::io::AsyncReadExt;
        loop {
            let n = file.read(&mut buffer).await?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    pub async fn sync_all(&self) -> Result<()> {
        self.sync_with_progress(|_, _| {}).await
    }

    pub async fn sync_with_progress<F>(&self, mut progress_callback: F) -> Result<()>
    where
        F: FnMut(u64, u64) + Send + Sync + 'static,
    {
        {
            let mut state = self.state.write().await;
            if state.is_syncing {
                return Err(anyhow!("Sync already in progress"));
            }
            state.is_syncing = true;
            state.last_sync_result = None;
        }

        let result = self.perform_sync(&mut progress_callback).await;

        let mut state = self.state.write().await;
        state.is_syncing = false;
        state.last_sync_result = Some(
            result
                .as_ref()
                .map(|_| ())
                .map_err(|e: &anyhow::Error| e.to_string()),
        );

        // Reload info on success
        if result.is_ok() {
            if let Ok(info) = self.load_asset_info("geoip.dat").await {
                state.geoip_info = Some(info);
            }
            if let Ok(info) = self.load_asset_info("geosite.dat").await {
                state.geosite_info = Some(info);
            }
        }

        result
    }

    async fn perform_sync<F>(&self, progress_callback: &mut F) -> Result<()>
    where
        F: FnMut(u64, u64) + Send + Sync + 'static,
    {
        // Define assets to download
        // Using optimized community rules
        let assets = vec![
            (
                "geoip.dat",
                "https://github.com/Loyalsoldier/v2ray-rules-dat/releases/latest/download/geoip.dat",
                "https://github.com/Loyalsoldier/v2ray-rules-dat/releases/latest/download/geoip.dat.sha256sum"
            ),
            (
                "geosite.dat",
                "https://github.com/Loyalsoldier/v2ray-rules-dat/releases/latest/download/geosite.dat",
                 "https://github.com/Loyalsoldier/v2ray-rules-dat/releases/latest/download/geosite.dat.sha256sum"
            )
        ];

        let mut total_bytes = 0;
        let mut downloaded_bytes = 0;

        // First pass: Get sizes
        for (_, url, _) in &assets {
            let resp = self.http_client.head(*url).send().await?;
            if let Some(len) = resp.headers().get(CONTENT_LENGTH) {
                if let Ok(val) = len.to_str() {
                    if let Ok(bytes) = val.parse::<u64>() {
                        total_bytes += bytes;
                    }
                }
            }
        }

        for (filename, url, checksum_url) in assets {
            let target_path = self.data_dir.join(filename);
            let tmp_path = self.data_dir.join(format!("{}.tmp", filename));

            // Download Checksum first
            let checksum_resp = self.http_client.get(checksum_url).send().await?;
            let checksum_content = checksum_resp.text().await?;
            let expected_checksum = checksum_content
                .split_whitespace()
                .next()
                .context("Invalid checksum format")?;

            // Check if we need to resume
            let mut file_start = 0;
            if tmp_path.exists() {
                file_start = fs::metadata(&tmp_path).await?.len();
            }

            let mut req = self.http_client.get(url);
            if file_start > 0 {
                req = req.header("Range", format!("bytes={}-", file_start));
            }

            let mut resp = req.send().await?;

            if resp.status().is_success() || resp.status() == StatusCode::PARTIAL_CONTENT {
                let mut file = fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(true)
                    .open(&tmp_path)
                    .await?;

                while let Some(chunk) = resp.chunk().await? {
                    file.write_all(&chunk).await?;
                    downloaded_bytes += chunk.len() as u64;
                    progress_callback(downloaded_bytes, total_bytes);
                }
            } else {
                return Err(anyhow!(
                    "Failed to download {}: Status {}",
                    filename,
                    resp.status()
                ));
            }

            // Verify checksum
            let calculated = self.calculate_file_checksum(&tmp_path).await?;
            if calculated != expected_checksum {
                // Corruption detected, remove tmp and fail
                fs::remove_file(&tmp_path).await?;
                return Err(anyhow!(
                    "Checksum mismatch for {}. Expected {}, got {}",
                    filename,
                    expected_checksum,
                    calculated
                ));
            }

            // Move to final
            fs::rename(&tmp_path, &target_path).await?;
            info!("Updated {}", filename);
        }

        Ok(())
    }
}
