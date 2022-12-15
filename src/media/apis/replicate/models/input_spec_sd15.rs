use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct InputStableDiffusion15 {
    pub prompt: String,
    pub negative_prompt: Option<String>,
    pub width: u16,
    pub height: u16,
    // pub prompt_strength: f32,
    pub num_outputs: u8,
    pub num_inference_steps: u16,
    pub guidance_scale: u8,
    pub scheduler: Option<String>,
    pub seed: Option<u64>,
}
