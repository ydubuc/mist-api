use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct InputSpec {
    pub prompt: String,
    pub width: u16,
    pub height: u16,
    pub number: u8,

    pub steps: Option<u8>,
    pub cfg_scale: Option<u8>,
    pub input_image_url: Option<String>,
    pub engine: Option<String>,
}
