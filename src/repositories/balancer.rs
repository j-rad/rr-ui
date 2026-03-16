//! Balancer repository - server-only module
#[cfg(feature = "server")]
use crate::api::balancer::Balancer;
use crate::db::DbClient;
use async_trait::async_trait;

#[async_trait]
pub trait BalancerRepository: Send + Sync {
    async fn list(&self) -> anyhow::Result<Vec<Balancer>>;
    async fn add(&self, balancer: Balancer) -> anyhow::Result<()>;
    async fn delete(&self, id: &str) -> anyhow::Result<()>;
}

pub struct SurrealBalancerRepository {
    db: DbClient,
}

impl SurrealBalancerRepository {
    pub fn new(db: DbClient) -> Self {
        Self { db }
    }
}

#[async_trait]
impl BalancerRepository for SurrealBalancerRepository {
    async fn list(&self) -> anyhow::Result<Vec<Balancer>> {
        let balancers: Vec<Balancer> = self.db.client.select("balancer").await?;
        Ok(balancers)
    }

    async fn add(&self, balancer: Balancer) -> anyhow::Result<()> {
        let _: Option<Balancer> = self
            .db
            .client
            .create(("balancer", &balancer.id))
            .content(balancer)
            .await?;
        Ok(())
    }

    async fn delete(&self, id: &str) -> anyhow::Result<()> {
        let _: Option<Balancer> = self.db.client.delete(("balancer", id)).await?;
        Ok(())
    }
}
