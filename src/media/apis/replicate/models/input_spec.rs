use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize)]
pub struct InputSpec {
    pub version: String,
    pub input: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_completed: Option<String>,
}
