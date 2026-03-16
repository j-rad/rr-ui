// src/adapters/repositories.rs - SurrealDB Repository Implementations
//
// These are adapters that implement the domain ports using SurrealDB.
// All SurrealDB-specific logic is contained here.

use crate::db::DbClient;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ports::{
    InboundRepository, OutboundRepository, SettingRepository, UserRepository,
};
use crate::models::{AllSetting, Client, ClientTraffic, Inbound, OutboundModel};
use async_trait::async_trait;
#[cfg(feature = "server")]
use std::str::FromStr;
#[cfg(feature = "server")]
use surrealdb::sql::Thing;

/// SurrealDB adapter for Inbound Repository
pub struct SurrealInboundRepository {
    db: DbClient,
}

impl SurrealInboundRepository {
    pub fn new(db: DbClient) -> Self {
        Self { db }
    }

    #[cfg(feature = "server")]
    fn parse_thing_id(id: &str) -> Option<Thing> {
        if id.contains(':') {
            Thing::from_str(id).ok()
        } else {
            Thing::from_str(&format!("inbound:{}", id)).ok()
        }
    }
}

#[async_trait]
impl InboundRepository for SurrealInboundRepository {
    async fn find_all(&self) -> DomainResult<Vec<Inbound<'static>>> {
        #[cfg(feature = "server")]
        {
            let result: Vec<Inbound<'static>> =
                self.db.client.select("inbound").await.map_err(|e| {
                    DomainError::RepositoryError {
                        message: format!("Failed to fetch inbounds: {}", e),
                    }
                })?;
            Ok(result)
        }
        #[cfg(not(feature = "server"))]
        {
            Ok(vec![])
        }
    }

    async fn find_by_id(&self, id: &str) -> DomainResult<Option<Inbound<'static>>> {
        #[cfg(feature = "server")]
        {
            let thing = Self::parse_thing_id(id).ok_or_else(|| DomainError::ValidationFailed {
                field: "id".to_string(),
                reason: "Invalid ID format".to_string(),
            })?;

            let result: Option<Inbound<'static>> = self
                .db
                .client
                .select(("inbound", thing.id.to_string()))
                .await
                .map_err(|e| DomainError::RepositoryError {
                    message: format!("Failed to fetch inbound by id: {}", e),
                })?;
            Ok(result)
        }
        #[cfg(not(feature = "server"))]
        {
            Ok(None)
        }
    }

    async fn find_by_tag(&self, tag: &str) -> DomainResult<Option<Inbound<'static>>> {
        #[cfg(feature = "server")]
        {
            let sql = format!("SELECT * FROM inbound WHERE tag = $tag");
            let mut result = self
                .db
                .client
                .query(&sql)
                .bind(("tag", tag.to_string()))
                .await
                .map_err(|e| DomainError::RepositoryError {
                    message: format!("Failed to query by tag: {}", e),
                })?;

            let inbounds: Vec<Inbound<'static>> =
                result.take(0).map_err(|e| DomainError::RepositoryError {
                    message: format!("Failed to extract query result: {}", e),
                })?;

            Ok(inbounds.into_iter().next())
        }
        #[cfg(not(feature = "server"))]
        {
            Ok(None)
        }
    }

    async fn find_enabled(&self) -> DomainResult<Vec<Inbound<'static>>> {
        #[cfg(feature = "server")]
        {
            let sql = "SELECT * FROM inbound WHERE enable = true";
            let mut result =
                self.db
                    .client
                    .query(sql)
                    .await
                    .map_err(|e| DomainError::RepositoryError {
                        message: format!("Failed to query enabled inbounds: {}", e),
                    })?;

            let inbounds: Vec<Inbound<'static>> =
                result.take(0).map_err(|e| DomainError::RepositoryError {
                    message: format!("Failed to extract enabled inbounds: {}", e),
                })?;

            Ok(inbounds)
        }
        #[cfg(not(feature = "server"))]
        {
            Ok(vec![])
        }
    }

    async fn create(&self, inbound: Inbound<'static>) -> DomainResult<Inbound<'static>> {
        #[cfg(feature = "server")]
        {
            let created: Option<Inbound<'static>> = self
                .db
                .client
                .create("inbound")
                .content(inbound.clone())
                .await
                .map_err(|e| DomainError::RepositoryError {
                    message: format!("Failed to create inbound: {}", e),
                })?;

            created.ok_or_else(|| DomainError::RepositoryError {
                message: "Create returned no result".to_string(),
            })
        }
        #[cfg(not(feature = "server"))]
        {
            Err(DomainError::RepositoryError {
                message: "Not implemented for client mode".to_string(),
            })
        }
    }

    async fn update(
        &self,
        id: &str,
        inbound: Inbound<'static>,
    ) -> DomainResult<Inbound<'static>> {
        #[cfg(feature = "server")]
        {
            let thing = Self::parse_thing_id(id).ok_or_else(|| DomainError::ValidationFailed {
                field: "id".to_string(),
                reason: "Invalid ID format".to_string(),
            })?;

            let updated: Option<Inbound<'static>> = self
                .db
                .client
                .update(("inbound", thing.id.to_string()))
                .content(inbound.clone())
                .await
                .map_err(|e| DomainError::RepositoryError {
                    message: format!("Failed to update inbound: {}", e),
                })?;

            updated.ok_or_else(|| DomainError::NotFound {
                resource: "Inbound".to_string(),
                id: id.to_string(),
            })
        }
        #[cfg(not(feature = "server"))]
        {
            Err(DomainError::RepositoryError {
                message: "Not implemented for client mode".to_string(),
            })
        }
    }

    async fn delete(&self, id: &str) -> DomainResult<Inbound<'static>> {
        #[cfg(feature = "server")]
        {
            let thing = Self::parse_thing_id(id).ok_or_else(|| DomainError::ValidationFailed {
                field: "id".to_string(),
                reason: "Invalid ID format".to_string(),
            })?;

            let deleted: Option<Inbound<'static>> = self
                .db
                .client
                .delete(("inbound", thing.id.to_string()))
                .await
                .map_err(|e| DomainError::RepositoryError {
                    message: format!("Failed to delete inbound: {}", e),
                })?;

            deleted.ok_or_else(|| DomainError::NotFound {
                resource: "Inbound".to_string(),
                id: id.to_string(),
            })
        }
        #[cfg(not(feature = "server"))]
        {
            Err(DomainError::RepositoryError {
                message: "Not implemented for client mode".to_string(),
            })
        }
    }

    async fn tag_exists(&self, tag: &str) -> DomainResult<bool> {
        Ok(self.find_by_tag(tag).await?.is_some())
    }

    async fn port_in_use(&self, port: u32) -> DomainResult<bool> {
        #[cfg(feature = "server")]
        {
            let sql = "SELECT * FROM inbound WHERE port = $port LIMIT 1";
            let mut result = self
                .db
                .client
                .query(sql)
                .bind(("port", port))
                .await
                .map_err(|e| DomainError::RepositoryError {
                    message: format!("Failed to check port usage: {}", e),
                })?;

            let inbounds: Vec<Inbound<'static>> =
                result.take(0).map_err(|e| DomainError::RepositoryError {
                    message: format!("Failed to extract port check result: {}", e),
                })?;

            Ok(!inbounds.is_empty())
        }
        #[cfg(not(feature = "server"))]
        {
            Ok(false)
        }
    }
}

/// SurrealDB adapter for Outbound Repository
pub struct SurrealOutboundRepository {
    db: DbClient,
}

impl SurrealOutboundRepository {
    pub fn new(db: DbClient) -> Self {
        Self { db }
    }
}

#[async_trait]
impl OutboundRepository for SurrealOutboundRepository {
    async fn find_all(&self) -> DomainResult<Vec<OutboundModel<'static>>> {
        #[cfg(feature = "server")]
        {
            let result: Vec<OutboundModel<'static>> =
                self.db.client.select("outbound").await.map_err(|e| {
                    DomainError::RepositoryError {
                        message: format!("Failed to fetch outbounds: {}", e),
                    }
                })?;
            Ok(result)
        }
        #[cfg(not(feature = "server"))]
        {
            Ok(vec![])
        }
    }

    async fn find_by_id(&self, id: &str) -> DomainResult<Option<OutboundModel<'static>>> {
        #[cfg(feature = "server")]
        {
            let result: Option<OutboundModel<'static>> =
                self.db.client.select(("outbound", id)).await.map_err(|e| {
                    DomainError::RepositoryError {
                        message: format!("Failed to fetch outbound: {}", e),
                    }
                })?;
            Ok(result)
        }
        #[cfg(not(feature = "server"))]
        {
            Ok(None)
        }
    }

    async fn find_by_tag(&self, tag: &str) -> DomainResult<Option<OutboundModel<'static>>> {
        #[cfg(feature = "server")]
        {
            let sql = "SELECT * FROM outbound WHERE tag = $tag";
            let mut result = self
                .db
                .client
                .query(sql)
                .bind(("tag", tag.to_string()))
                .await
                .map_err(|e| DomainError::RepositoryError {
                    message: format!("Failed to query outbound by tag: {}", e),
                })?;

            let outbounds: Vec<OutboundModel<'static>> =
                result.take(0).map_err(|e| DomainError::RepositoryError {
                    message: format!("Failed to extract outbound: {}", e),
                })?;

            Ok(outbounds.into_iter().next())
        }
        #[cfg(not(feature = "server"))]
        {
            Ok(None)
        }
    }

    async fn find_enabled(&self) -> DomainResult<Vec<OutboundModel<'static>>> {
        #[cfg(feature = "server")]
        {
            let sql = "SELECT * FROM outbound WHERE enable = true";
            let mut result =
                self.db
                    .client
                    .query(sql)
                    .await
                    .map_err(|e| DomainError::RepositoryError {
                        message: format!("Failed to query enabled outbounds: {}", e),
                    })?;

            let outbounds: Vec<OutboundModel<'static>> =
                result.take(0).map_err(|e| DomainError::RepositoryError {
                    message: format!("Failed to extract enabled outbounds: {}", e),
                })?;

            Ok(outbounds)
        }
        #[cfg(not(feature = "server"))]
        {
            Ok(vec![])
        }
    }
}

/// SurrealDB adapter for User Repository
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
    async fn get_traffic(&self, email: &str) -> DomainResult<Option<ClientTraffic>> {
        #[cfg(feature = "server")]
        {
            let sql = "SELECT * FROM client_traffic WHERE email = $email";
            let mut result = self
                .db
                .client
                .query(sql)
                .bind(("email", email.to_string()))
                .await
                .map_err(|e| DomainError::RepositoryError {
                    message: format!("Failed to fetch traffic: {}", e),
                })?;

            let stats: Vec<ClientTraffic> =
                result.take(0).map_err(|e| DomainError::RepositoryError {
                    message: format!("Failed to extract traffic data: {}", e),
                })?;

            Ok(stats.into_iter().next())
        }
        #[cfg(not(feature = "server"))]
        {
            Ok(None)
        }
    }

    async fn reset_traffic(&self, email: &str) -> DomainResult<()> {
        #[cfg(feature = "server")]
        {
            let sql = "UPDATE client_traffic SET up = 0, down = 0, total = 0 WHERE email = $email";
            self.db
                .client
                .query(sql)
                .bind(("email", email.to_string()))
                .await
                .map_err(|e| DomainError::RepositoryError {
                    message: format!("Failed to reset traffic: {}", e),
                })?;
            Ok(())
        }
        #[cfg(not(feature = "server"))]
        {
            Ok(())
        }
    }

    async fn get_all_traffic(&self) -> DomainResult<Vec<ClientTraffic>> {
        #[cfg(feature = "server")]
        {
            let result: Vec<ClientTraffic> =
                self.db.client.select("client_traffic").await.map_err(|e| {
                    DomainError::RepositoryError {
                        message: format!("Failed to fetch all traffic: {}", e),
                    }
                })?;
            Ok(result)
        }
        #[cfg(not(feature = "server"))]
        {
            Ok(vec![])
        }
    }

    async fn update_traffic(&self, email: &str, up: i64, down: i64) -> DomainResult<()> {
        #[cfg(feature = "server")]
        {
            let sql = "UPDATE client_traffic SET up += $up, down += $down, total = up + down WHERE email = $email";
            self.db
                .client
                .query(sql)
                .bind(("email", email.to_string()))
                .bind(("up", up))
                .bind(("down", down))
                .await
                .map_err(|e| DomainError::RepositoryError {
                    message: format!("Failed to update traffic: {}", e),
                })?;
            Ok(())
        }
        #[cfg(not(feature = "server"))]
        {
            Ok(())
        }
    }

    async fn find_client_by_email(&self, email: &str) -> DomainResult<Option<(Client, String)>> {
        #[cfg(feature = "server")]
        {
            // This requires querying inbounds to find the client
            let sql = "SELECT * FROM inbound WHERE enable = true";
            let mut result =
                self.db
                    .client
                    .query(sql)
                    .await
                    .map_err(|e| DomainError::RepositoryError {
                        message: format!("Failed to query inbounds for client: {}", e),
                    })?;

            let inbounds: Vec<Inbound<'static>> =
                result.take(0).map_err(|e| DomainError::RepositoryError {
                    message: format!("Failed to extract inbounds: {}", e),
                })?;

            for inbound in inbounds {
                if let Some(clients) = inbound.settings.clients() {
                    if let Some(client) = clients.iter().find(|c| c.email.as_deref() == Some(email))
                    {
                        return Ok(Some((client.clone(), inbound.tag.into_owned())));
                    }
                }
            }

            Ok(None)
        }
        #[cfg(not(feature = "server"))]
        {
            Ok(None)
        }
    }

    async fn email_is_unique(
        &self,
        email: &str,
        exclude_inbound: Option<&str>,
    ) -> DomainResult<bool> {
        #[cfg(feature = "server")]
        {
            let sql = "SELECT * FROM inbound WHERE enable = true";
            let mut result =
                self.db
                    .client
                    .query(sql)
                    .await
                    .map_err(|e| DomainError::RepositoryError {
                        message: format!("Failed to check email uniqueness: {}", e),
                    })?;

            let inbounds: Vec<Inbound<'static>> =
                result.take(0).map_err(|e| DomainError::RepositoryError {
                    message: format!("Failed to extract inbounds for uniqueness check: {}", e),
                })?;

            for inbound in inbounds {
                // Skip excluded inbound
                if let Some(excluded) = exclude_inbound {
                    if inbound.tag == excluded {
                        continue;
                    }
                }

                if let Some(clients) = inbound.settings.clients() {
                    if clients.iter().any(|c| c.email.as_deref() == Some(email)) {
                        return Ok(false);
                    }
                }
            }

            Ok(true)
        }
        #[cfg(not(feature = "server"))]
        {
            Ok(true)
        }
    }
}

/// SurrealDB adapter for Setting Repository
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
    async fn get(&self) -> DomainResult<Option<AllSetting>> {
        #[cfg(feature = "server")]
        {
            let sql = "SELECT * FROM setting LIMIT 1";
            let mut result =
                self.db
                    .client
                    .query(sql)
                    .await
                    .map_err(|e| DomainError::RepositoryError {
                        message: format!("Failed to fetch settings: {}", e),
                    })?;

            let settings: Vec<AllSetting> =
                result.take(0).map_err(|e| DomainError::RepositoryError {
                    message: format!("Failed to extract settings: {}", e),
                })?;

            Ok(settings.into_iter().next())
        }
        #[cfg(not(feature = "server"))]
        {
            Ok(None)
        }
    }

    async fn save(&self, settings: AllSetting) -> DomainResult<()> {
        #[cfg(feature = "server")]
        {
            let sql = "DELETE FROM setting; CREATE setting CONTENT $data;";
            let data =
                serde_json::to_value(&settings).map_err(|e| DomainError::ConfigurationError {
                    message: format!("Failed to serialize settings: {}", e),
                })?;

            self.db
                .client
                .query(sql)
                .bind(("data", data))
                .await
                .map_err(|e| DomainError::RepositoryError {
                    message: format!("Failed to save settings: {}", e),
                })?;

            Ok(())
        }
        #[cfg(not(feature = "server"))]
        {
            Ok(())
        }
    }

    async fn update_core_type(&self, core_type: String) -> DomainResult<()> {
        #[cfg(feature = "server")]
        {
            let sql = "UPDATE setting SET core_type = $core_type";
            self.db
                .client
                .query(sql)
                .bind(("core_type", core_type))
                .await
                .map_err(|e| DomainError::RepositoryError {
                    message: format!("Failed to update core type: {}", e),
                })?;

            Ok(())
        }
        #[cfg(not(feature = "server"))]
        {
            Ok(())
        }
    }
}
