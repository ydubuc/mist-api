use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DalleGenerateImageResponse {
    pub created: u64,
    pub data: Vec<DalleData>,
}

#[derive(Debug, Deserialize)]
pub struct DalleData {
    pub url: String,
}
