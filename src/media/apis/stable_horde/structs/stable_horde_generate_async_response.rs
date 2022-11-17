use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct StableHordeGenerateAsyncResponse {
    pub id: String,
    pub message: Option<String>,
}
