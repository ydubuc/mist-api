use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct InputSpec {
    pub style: u8,
    pub prompt: String,
    pub target_image_weight: Option<f64>,
    pub width: Option<u16>,
    pub height: Option<u16>,
}
