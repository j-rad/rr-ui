//! Routing repository - server-only module
#[cfg(feature = "server")]
use crate::api::routing::RoutingRule;
use crate::db::DbClient;
use async_trait::async_trait;

#[async_trait]
pub trait RoutingRepository: Send + Sync {
    async fn list(&self) -> anyhow::Result<Vec<RoutingRule>>;
    async fn add(&self, rule: RoutingRule) -> anyhow::Result<()>;
    async fn delete(&self, id: &str) -> anyhow::Result<()>;
}

pub struct SurrealRoutingRepository {
    db: DbClient,
}

impl SurrealRoutingRepository {
    pub fn new(db: DbClient) -> Self {
        Self { db }
    }
}

#[async_trait]
impl RoutingRepository for SurrealRoutingRepository {
    async fn list(&self) -> anyhow::Result<Vec<RoutingRule>> {
        let rules: Vec<RoutingRule> = self.db.client.select("routing_rule").await?;
        Ok(rules)
    }

    async fn add(&self, rule: RoutingRule) -> anyhow::Result<()> {
        let _: Option<RoutingRule> = self
            .db
            .client
            .create(("routing_rule", &rule.id))
            .content(rule)
            .await?;
        Ok(())
    }

    async fn delete(&self, id: &str) -> anyhow::Result<()> {
        let _: Option<RoutingRule> = self.db.client.delete(("routing_rule", id)).await?;
        Ok(())
    }
}
