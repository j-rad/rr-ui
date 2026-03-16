// src/db.rs
use anyhow::Result;
use std::sync::Arc;

use std::path::PathBuf;
#[cfg(feature = "server")]
use surrealdb::Surreal;
#[cfg(feature = "server")]
use surrealdb::engine::local::{Db, SurrealKv};

#[cfg(feature = "server")]
#[derive(Clone)]
pub struct SurrealDbClient {
    pub client: Arc<Surreal<Db>>,
}

#[cfg(feature = "server")]
impl SurrealDbClient {
    pub async fn init(path: &str) -> Result<Self> {
        let db_path = Self::get_db_path(path);

        // Ensure database directory exists
        tokio::fs::create_dir_all(&db_path).await?;

        let client = Surreal::new::<SurrealKv>(db_path.to_string_lossy().to_string())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to open SurrealKV at {:?}: {}", db_path, e))?;
        client.use_ns("xui").use_db("panel").await?;
        let db_client = Self {
            client: Arc::new(client),
        };
        db_client.setup_schema().await?;
        Ok(db_client)
    }

    fn get_db_path(default_name: &str) -> PathBuf {
        // High-priority system path for production service
        let system_base = PathBuf::from("/etc/rr-ui");
        let system_db = system_base.join("db");

        // Check if we have write access to the system path
        if system_base.exists() {
            // Check if directory is writable or if we are root
            #[cfg(unix)]
            {
                use nix::unistd::{AccessFlags, access};
                if access(&system_base, AccessFlags::W_OK).is_ok() {
                    return system_db;
                }
            }
            #[cfg(not(unix))]
            {
                return system_db;
            }
        }

        // Check for XDG_DATA_HOME (good for non-root users)
        if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
            let mut path = PathBuf::from(xdg_data);
            path.push("rr-ui");
            path.push(default_name);
            return path;
        }

        // Fallback to local execution path if /etc/rr-ui doesn't exist (dev mode)
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        current_dir.join("data").join(default_name)
    }

    async fn setup_schema(&self) -> Result<()> {
        // Inbounds
        let _ = self.client.query("DEFINE TABLE inbound SCHEMALESS").await?;
        let _ = self
            .client
            .query("DEFINE FIELD tag ON TABLE inbound TYPE string")
            .await?;

        // Client Traffic (users)
        let _ = self
            .client
            .query("DEFINE TABLE client_traffic SCHEMALESS")
            .await?;
        let _ = self
            .client
            .query("DEFINE FIELD email ON TABLE client_traffic TYPE string")
            .await?;

        // Settings
        let _ = self.client.query("DEFINE TABLE setting SCHEMALESS").await?;
        let _ = self
            .client
            .query("DEFINE FIELD core_path ON TABLE setting TYPE option<string>")
            .await?;

        // Routing & Balancers
        let _ = self
            .client
            .query("DEFINE TABLE routing_rule SCHEMALESS")
            .await?;
        let _ = self
            .client
            .query("DEFINE TABLE balancer SCHEMALESS")
            .await?;

        Ok(())
    }

    pub fn get_client(&self) -> Arc<Surreal<Db>> {
        self.client.clone()
    }
}

#[cfg(not(feature = "server"))]
#[derive(Clone)]
pub struct FileDbClient {
    pub path: String,
}

#[cfg(not(feature = "server"))]
impl FileDbClient {
    pub async fn init(path: &str) -> Result<Self> {
        // Ensure file exists or create it
        if !std::path::Path::new(path).exists() {
            tokio::fs::write(path, "{}").await?;
        }
        Ok(Self {
            path: path.to_string(),
        })
    }
}

#[cfg(feature = "server")]
pub type DbClient = SurrealDbClient;

#[cfg(not(feature = "server"))]
pub type DbClient = FileDbClient;
