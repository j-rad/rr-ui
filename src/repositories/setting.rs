use crate::db::DbClient;
use crate::models::AllSetting;
use anyhow::Result;

use async_trait::async_trait;

#[async_trait]
pub trait SettingOps {
    async fn get(db: &DbClient) -> Result<Option<AllSetting>>;
    async fn save(&self, db: &DbClient) -> Result<()>;
}

#[async_trait]
impl SettingOps for AllSetting {
    async fn get(db: &DbClient) -> Result<Option<AllSetting>> {
        let settings: Vec<AllSetting> = db.client.select("setting").await?;
        Ok(settings.into_iter().next())
    }

    async fn save(&self, db: &DbClient) -> Result<()> {
        let current: Vec<AllSetting> = db.client.select("setting").await?;
        if current.is_empty() {
            let _: Option<AllSetting> = db.client.create("setting").content(self.clone()).await?;
        } else {
            let _: Vec<AllSetting> = db.client.update("setting").content(self.clone()).await?;
        }
        Ok(())
    }
}

#[async_trait]
pub trait SettingRepository: Send + Sync {
    async fn get(&self) -> Result<Option<AllSetting>>;
    async fn save(&self, settings: AllSetting) -> Result<()>;
}

pub struct SurrealSettingRepository {
    db: DbClient,
}

impl SurrealSettingRepository {
    pub fn new(db: DbClient) -> Self {
        Self { db }
    }
}

#[async_trait]
impl SettingRepository for SurrealSettingRepository {
    async fn get(&self) -> Result<Option<AllSetting>> {
        let settings: Vec<AllSetting> = self.db.client.select("setting").await?;
        Ok(settings.into_iter().next())
    }

    async fn save(&self, settings: AllSetting) -> Result<()> {
        // Treat 'setting' table as a singleton; update all records (should be only one)
        // If empty, create one.
        let current: Vec<AllSetting> = self.db.client.select("setting").await?;
        if current.is_empty() {
            let _: Option<AllSetting> = self.db.client.create("setting").content(settings).await?;
        } else {
            let _: Vec<AllSetting> = self.db.client.update("setting").content(settings).await?;
        }
        Ok(())
    }
}
