use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize)]
pub struct InputSpec {
    pub version: String,
    pub input: Value,
    pub webhook_completed: Option<String>,
}
