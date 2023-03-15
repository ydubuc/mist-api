use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct InputModal {
    pub request_id: String,
    pub prompt: String,
    pub negative_prompt: Option<String>,
    pub width: u16,
    pub height: u16,
    pub number: u8,
    pub steps: u16,
    pub cfg_scale: u8,
    pub callback_url: String,
}
