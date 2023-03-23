use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct InputStableDiffusion21 {
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negative_prompt: Option<String>,
    pub width: u16,
    pub height: u16,
    // pub prompt_strength: f32,
    pub num_outputs: u8,
    pub num_inference_steps: u16,
    pub guidance_scale: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduler: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
}
