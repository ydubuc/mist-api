use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LabmlGetRequestResponse {
    pub is_success: bool,
    pub is_completed: bool,
    pub eta: f32,
    pub queue_position: i32,
    pub job_id: String,
    pub nsfw_triggered: bool,
    pub images: Vec<LabmlImage>,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct LabmlImage {
    pub image: String,
    pub thumbnail: String,
}
