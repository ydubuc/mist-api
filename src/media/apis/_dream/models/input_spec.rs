use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct InputSpec {
    pub style: u8,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_image_weight: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u16>,
}
