// src/domain/graph_schema.rs
//! SurrealDB Graph Schema
//!
//! Defines the multi-tenant hierarchy:
//! Admin -> Reseller -> Group -> Node -> User

use serde::{Deserialize, Serialize};

/// Admin entity (top level)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Admin {
    pub id: String,
    pub username: String,
    pub email: String,
    pub created_at: i64,
    pub permissions: AdminPermissions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminPermissions {
    pub can_create_resellers: bool,
    pub can_view_all_stats: bool,
    pub can_manage_billing: bool,
}

/// Reseller entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reseller {
    pub id: String,
    pub name: String,
    pub admin_id: String,
    pub quota: ResellerQuota,
    pub billing: BillingInfo,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResellerQuota {
    pub max_groups: u32,
    pub max_nodes: u32,
    pub max_users: u32,
    pub max_bandwidth_gbps: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingInfo {
    pub plan: String,
    pub monthly_cost: f64,
    pub currency: String,
    pub next_billing_date: i64,
}

/// Group entity (collection of nodes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub reseller_id: String,
    pub region: String,
    pub created_at: i64,
}

/// Node entity (edge server)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub name: String,
    pub group_id: String,
    pub address: String,
    pub port: u16,
    pub status: NodeStatus,
    pub health: NodeHealth,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeStatus {
    Active,
    Inactive,
    Maintenance,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeHealth {
    pub cpu_percent: f32,
    pub memory_percent: f32,
    pub disk_percent: f32,
    pub uptime_seconds: u64,
    pub last_check: i64,
}

/// User entity (end client)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub node_id: String,
    pub quota: UserQuota,
    pub traffic: TrafficStats,
    pub created_at: i64,
    pub expires_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserQuota {
    pub total_gb: u64,
    pub used_gb: u64,
    pub reset_day: u8, // 1-31
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficStats {
    pub upload_bytes: u64,
    pub download_bytes: u64,
    pub last_active: i64,
}

/// Graph relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Relationship {
    Manages { from: String, to: String },
    Owns { from: String, to: String },
    Contains { from: String, to: String },
    Serves { from: String, to: String },
}

/// SurrealDB schema initialization
pub const SCHEMA_INIT: &str = r#"
-- Define tables
DEFINE TABLE admin SCHEMAFULL;
DEFINE TABLE reseller SCHEMAFULL;
DEFINE TABLE group SCHEMAFULL;
DEFINE TABLE node SCHEMAFULL;
DEFINE TABLE user SCHEMAFULL;

-- Define relationships
DEFINE TABLE manages SCHEMAFULL;
DEFINE TABLE owns SCHEMAFULL;
DEFINE TABLE contains SCHEMAFULL;
DEFINE TABLE serves SCHEMAFULL;

-- Admin fields
DEFINE FIELD username ON admin TYPE string;
DEFINE FIELD email ON admin TYPE string;
DEFINE FIELD created_at ON admin TYPE int;
DEFINE FIELD permissions ON admin TYPE object;

-- Reseller fields
DEFINE FIELD name ON reseller TYPE string;
DEFINE FIELD admin_id ON reseller TYPE string;
DEFINE FIELD quota ON reseller TYPE object;
DEFINE FIELD billing ON reseller TYPE object;
DEFINE FIELD created_at ON reseller TYPE int;

-- Group fields
DEFINE FIELD name ON group TYPE string;
DEFINE FIELD reseller_id ON group TYPE string;
DEFINE FIELD region ON group TYPE string;
DEFINE FIELD created_at ON group TYPE int;

-- Node fields
DEFINE FIELD name ON node TYPE string;
DEFINE FIELD group_id ON node TYPE string;
DEFINE FIELD address ON node TYPE string;
DEFINE FIELD port ON node TYPE int;
DEFINE FIELD status ON node TYPE string;
DEFINE FIELD health ON node TYPE object;
DEFINE FIELD created_at ON node TYPE int;

-- User fields
DEFINE FIELD email ON user TYPE string;
DEFINE FIELD node_id ON user TYPE string;
DEFINE FIELD quota ON user TYPE object;
DEFINE FIELD traffic ON user TYPE object;
DEFINE FIELD created_at ON user TYPE int;
DEFINE FIELD expires_at ON user TYPE int;

-- Relationship fields
DEFINE FIELD in ON manages TYPE record(admin);
DEFINE FIELD out ON manages TYPE record(reseller);

DEFINE FIELD in ON owns TYPE record(reseller);
DEFINE FIELD out ON owns TYPE record(group);

DEFINE FIELD in ON contains TYPE record(group);
DEFINE FIELD out ON contains TYPE record(node);

DEFINE FIELD in ON serves TYPE record(node);
DEFINE FIELD out ON serves TYPE record(user);

-- Indexes
DEFINE INDEX admin_email ON admin FIELDS email UNIQUE;
DEFINE INDEX reseller_admin ON reseller FIELDS admin_id;
DEFINE INDEX group_reseller ON group FIELDS reseller_id;
DEFINE INDEX node_group ON node FIELDS group_id;
DEFINE INDEX user_node ON user FIELDS node_id;
DEFINE INDEX user_email ON user FIELDS email UNIQUE;
"#;

/// Migration helper
pub struct GraphMigration;

impl GraphMigration {
    pub fn get_init_script() -> &'static str {
        SCHEMA_INIT
    }

    /// Get query to create relationship
    pub fn create_relationship(rel: &Relationship) -> String {
        match rel {
            Relationship::Manages { from, to } => {
                format!("RELATE {}->manages->{}", from, to)
            }
            Relationship::Owns { from, to } => {
                format!("RELATE {}->owns->{}", from, to)
            }
            Relationship::Contains { from, to } => {
                format!("RELATE {}->contains->{}", from, to)
            }
            Relationship::Serves { from, to } => {
                format!("RELATE {}->serves->{}", from, to)
            }
        }
    }

    /// Query to get all nodes under a reseller
    pub fn get_reseller_nodes(reseller_id: &str) -> String {
        format!(
            "SELECT * FROM node WHERE group_id IN (SELECT id FROM group WHERE reseller_id = '{}')",
            reseller_id
        )
    }

    /// Query to get total traffic for a reseller
    pub fn get_reseller_traffic(reseller_id: &str) -> String {
        format!(
            r#"
            SELECT 
                math::sum(traffic.upload_bytes) AS total_upload,
                math::sum(traffic.download_bytes) AS total_download
            FROM user 
            WHERE node_id IN (
                SELECT id FROM node WHERE group_id IN (
                    SELECT id FROM group WHERE reseller_id = '{}'
                )
            )
            "#,
            reseller_id
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relationship_query() {
        let rel = Relationship::Manages {
            from: "admin:1".into(),
            to: "reseller:1".into(),
        };
        let query = GraphMigration::create_relationship(&rel);
        assert!(query.contains("->manages->"));
    }

    #[test]
    fn test_schema_not_empty() {
        assert!(!SCHEMA_INIT.is_empty());
        assert!(SCHEMA_INIT.contains("DEFINE TABLE"));
    }
}
