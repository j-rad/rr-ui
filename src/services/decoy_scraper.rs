// src/services/decoy_scraper.rs
use crate::db::DbClient;
use crate::models::AllSetting;
use crate::repositories::setting::SettingOps;
use log::{error, info, warn};
use std::collections::HashMap;
use std::time::Duration;

pub struct DecoyScraper {
    db: DbClient,
    targets: Vec<String>,
}

impl DecoyScraper {
    pub fn new(db: DbClient) -> Self {
        Self {
            db,
            targets: vec![
                "https://www.baidu.com".to_string(),
                "https://www.taobao.com".to_string(),
                "https://www.qq.com".to_string(),
                "https://www.jd.com".to_string(),
                "https://www.sina.com.cn".to_string(),
            ],
        }
    }

    pub async fn run(self) {
        let mut interval = tokio::time::interval(Duration::from_secs(3600)); // Scrape every hour

        loop {
            interval.tick().await;
            info!("Starting decoy header scraping...");
            if let Err(e) = self.scrape_all().await {
                error!("Decoy scraping failed: {}", e);
            }
        }
    }

    async fn scrape_all(&self) -> anyhow::Result<()> {
        let mut scraped_headers = HashMap::new();
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()?;

        for target in &self.targets {
            match client.get(target).send().await {
                Ok(resp) => {
                    let mut headers_map = HashMap::new();
                    for (name, value) in resp.headers() {
                        if let Ok(val_str) = value.to_str() {
                            headers_map.insert(name.to_string(), val_str.to_string());
                        }
                    }

                    if let Ok(json) = serde_json::to_string(&headers_map) {
                        scraped_headers.insert(target.clone(), json);
                        info!("Scraped headers from {}", target);
                    }
                }
                Err(e) => {
                    warn!("Failed to scrape {}: {}", target, e);
                }
            }
        }

        if !scraped_headers.is_empty() {
            // Update settings in DB
            if let Some(mut settings) = AllSetting::get(&self.db).await? {
                settings.scraped_decoy_headers = scraped_headers;
                settings.save(&self.db).await?;
                info!("Decoy headers updated in database");
            }
        }

        Ok(())
    }
}
