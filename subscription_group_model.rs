// Subscription Group Model - appended to models.rs

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionGroup {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "server", serde(alias = "id"))]
    pub id: Option<IdType>,
    pub remark: String,
    pub inbound_ids: Vec<String>,
    pub is_enabled: bool,
    pub expiry_time: i64,
    #[serde(default)]
    pub total_traffic: i64,
    #[serde(default)]
    pub used_traffic: i64,
    #[serde(default)]
    pub created_at: i64,
}

impl Default for SubscriptionGroup {
    fn default() -> Self {
        Self {
            id: None,
            remark: String::new(),
            inbound_ids: Vec::new(),
            is_enabled: true,
            expiry_time: 0,
            total_traffic: 0,
            used_traffic: 0,
            created_at: chrono::Utc::now().timestamp(),
        }
    }
}
