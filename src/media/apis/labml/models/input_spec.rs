use serde::{Deserialize, Serialize};

// https://promptart.labml.ai/docs
#[derive(Debug, Serialize, Deserialize)]
pub struct InputSpec {
    pub api_token: String,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negative_prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n_steps: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling_method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_strength: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seeds: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mask: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_strength: Option<f32>,
}
