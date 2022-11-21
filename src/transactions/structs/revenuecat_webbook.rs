use serde::{Deserialize, Serialize};

// https://www.revenuecat.com/docs/webhooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueCatWebhook {
    pub event: serde_json::Value,
    pub api_version: String,
}
