use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct ReplicatePredictionsResponse {
    pub id: String,
    pub version: String,
    pub urls: ReplicateUrls,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub status: String,
    pub input: Value,
    pub output: Option<Vec<String>>,
    pub error: Option<String>,
    pub logs: Option<String>,
    pub metrics: ReplicateMetrics,
}

#[derive(Debug, Deserialize)]
pub struct ReplicateUrls {
    pub get: String,
    pub cancel: String,
}

#[derive(Debug, Deserialize)]
pub struct ReplicateMetrics {
    pub predict_time: Option<f32>,
}
