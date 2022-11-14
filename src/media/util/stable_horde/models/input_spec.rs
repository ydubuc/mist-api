use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct InputSpec {
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<InputSpecParams>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsfw: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trusted_workers: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub censor_nsfw: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workers: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_processing: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_mask: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InputSpecParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cfg_scale: Option<i8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub denoising_strength: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed_variation: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_gfpgan: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub karras: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_real_esrgan: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_ldsr: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_upscaling: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steps: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u8>,
}
