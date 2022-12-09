use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LabmlGenerateResponse {
    pub is_success: bool,
    pub eta: f32,
    pub queue_position: i32,
    pub job_id: String,
    pub message: String,
}
