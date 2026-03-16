use crate::db::DbClient;
use crate::models::ClientTraffic;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn get_traffic(&self, email: &str) -> Result<Option<ClientTraffic>>;
    async fn reset_traffic(&self, email: &str) -> Result<()>;
}

pub struct SurrealUserRepository {
    db: DbClient,
}

impl SurrealUserRepository {
    pub fn new(db: DbClient) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UserRepository for SurrealUserRepository {
    async fn get_traffic(&self, email: &str) -> Result<Option<ClientTraffic>> {
        #[cfg(feature = "server")]
        {
            let sql = format!("SELECT * FROM client_traffic WHERE email = '{}'", email);
            let mut result = self.db.client.query(&sql).await?;
            let stats: Vec<ClientTraffic> = result.take(0)?;
            Ok(stats.into_iter().next())
        }
        #[cfg(not(feature = "server"))]
        {
            Ok(None)
        }
    }

    async fn reset_traffic(&self, email: &str) -> Result<()> {
        #[cfg(feature = "server")]
        {
            let sql = format!(
                "UPDATE client_traffic SET up = 0, down = 0 WHERE email = '{}'",
                email
            );
            let _ = self.db.client.query(&sql).await?;
            Ok(())
        }
        #[cfg(not(feature = "server"))]
        {
            Ok(())
        }
    }
}
