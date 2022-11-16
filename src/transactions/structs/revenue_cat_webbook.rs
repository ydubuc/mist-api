use serde::{Deserialize, Serialize};

// https://www.revenuecat.com/docs/webhooks
#[derive(Debug, Serialize, Deserialize)]
pub struct RevenueCatWebhook {
    pub event: sqlx::types::JsonValue,
    pub api_version: String,
}
