use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct InputSpecOpenjourney {
    pub prompt: String,
    pub width: u16,
    pub height: u16,
    pub num_outputs: u8,
    pub num_inference_steps: u16,
    pub guidance_scale: u8,
    pub seed: Option<u64>,
}
