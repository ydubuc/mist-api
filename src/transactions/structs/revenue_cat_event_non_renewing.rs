use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RevenueCatWebhookEventNonRenewing {
    #[serde(rename = "type")]
    pub event_type: String,
    pub id: String,
    pub app_id: String,
    pub event_timestamp_ms: i64,
    pub app_user_id: String,
    pub original_app_user_id: String,
    pub aliases: Vec<String>,
    pub product_id: String,
}
