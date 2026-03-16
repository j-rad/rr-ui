use crate::db::DbClient;
use crate::models::{Inbound, OutboundModel};
use anyhow::Result;
use async_trait::async_trait;
#[cfg(feature = "server")]
use std::str::FromStr;
#[cfg(feature = "server")]
use surrealdb::sql::Thing;

#[async_trait]
pub trait InboundRepository: Send + Sync {
    async fn list(&self) -> Result<Vec<Inbound<'static>>>;
    async fn get_outbounds(&self) -> Result<Vec<OutboundModel<'static>>>;
    async fn create(&self, inbound: Inbound<'static>)
        -> Result<Option<Inbound<'static>>>;
    async fn update(
        &self,
        id: &str,
        inbound: Inbound<'static>,
    ) -> Result<Option<Inbound<'static>>>;
    async fn delete(&self, id: &str) -> Result<Option<Inbound<'static>>>;
}

pub struct SurrealInboundRepository {
    db: DbClient,
}

impl SurrealInboundRepository {
    pub fn new(db: DbClient) -> Self {
        Self { db }
    }
}

#[async_trait]
impl InboundRepository for SurrealInboundRepository {
    async fn list(&self) -> Result<Vec<Inbound<'static>>> {
        #[cfg(feature = "server")]
        {
            Ok(self.db.client.select("inbound").await?)
        }
        #[cfg(not(feature = "server"))]
        {
            Ok(vec![])
        }
    }

    async fn get_outbounds(&self) -> Result<Vec<OutboundModel<'static>>> {
        #[cfg(feature = "server")]
        {
            Ok(self.db.client.select("outbound").await?)
        }
        #[cfg(not(feature = "server"))]
        {
            Ok(vec![])
        }
    }

    async fn create(
        &self,
        inbound: Inbound<'static>,
    ) -> Result<Option<Inbound<'static>>> {
        #[cfg(feature = "server")]
        {
            Ok(self
                .db
                .client
                .create("inbound")
                .content(inbound.clone())
                .await?)
        }
        #[cfg(not(feature = "server"))]
        {
            Ok(None)
        }
    }

    async fn update(
        &self,
        id: &str,
        inbound: Inbound<'static>,
    ) -> Result<Option<Inbound<'static>>> {
        #[cfg(feature = "server")]
        {
            let tid = if id.contains(":") {
                Thing::from_str(id).ok()
            } else {
                Thing::from_str(&format!("inbound:{}", id)).ok()
            };

            if let Some(thing) = tid {
                Ok(self
                    .db
                    .client
                    .update(("inbound", thing.id.to_string()))
                    .content(inbound.clone())
                    .await?)
            } else {
                Ok(None)
            }
        }
        #[cfg(not(feature = "server"))]
        {
            Ok(None)
        }
    }

    async fn delete(&self, id: &str) -> Result<Option<Inbound<'static>>> {
        #[cfg(feature = "server")]
        {
            let tid = if id.contains(":") {
                Thing::from_str(id).ok()
            } else {
                Thing::from_str(&format!("inbound:{}", id)).ok()
            };

            if let Some(thing) = tid {
                Ok(self
                    .db
                    .client
                    .delete(("inbound", thing.id.to_string()))
                    .await?)
            } else {
                Ok(None)
            }
        }
        #[cfg(not(feature = "server"))]
        {
            Ok(None)
        }
    }
}
