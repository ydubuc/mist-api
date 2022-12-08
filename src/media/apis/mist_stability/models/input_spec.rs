use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct InputSpec {
    pub prompt: String,
    pub width: u16,
    pub height: u16,
    pub number: u8,
    pub steps: Option<u8>,
    pub engine: Option<String>,
}
