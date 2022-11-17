use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct InputSpec {
    pub prompt: String,
    pub n: u8,
    pub size: String,
    pub response_format: String,
}
