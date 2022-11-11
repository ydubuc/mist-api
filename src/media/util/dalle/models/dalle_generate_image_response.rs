use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DalleGenerateImageResponse {
    #[serde(rename(deserialize = "created"))]
    pub created: u64,
    #[serde(rename(deserialize = "data"))]
    pub data: Vec<DalleData>,
}

#[derive(Debug, Deserialize)]
pub struct DalleData {
    #[serde(rename(deserialize = "url"))]
    pub url: String,
}
