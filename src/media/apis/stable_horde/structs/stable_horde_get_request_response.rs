use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct StableHordeGetRequestResponse {
    pub finished: u16,
    pub processing: u16,
    pub restarted: u16,
    pub waiting: u16,
    pub done: bool,
    pub faulted: bool,
    pub wait_time: u32,
    pub queue_position: u32,
    pub kudos: f32,
    pub is_possible: bool,
    pub generations: Option<Vec<StableHordeGeneration>>,
}

#[derive(Debug, Deserialize)]
pub struct StableHordeGeneration {
    pub worker_id: String,
    pub worker_name: String,
    pub model: String,
    pub img: String,
    pub seed: String,
}
