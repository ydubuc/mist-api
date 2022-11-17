use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DalleGenerateImageResponse {
    #[serde(rename(deserialize = "created"))]
    pub created: u64,
    #[serde(rename(deserialize = "data"))]
    pub data: Vec<DalleDataBase64Json>,
}

#[derive(Debug, Deserialize)]
pub struct DalleDataUrl {
    #[serde(rename(deserialize = "url"))]
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct DalleDataBase64Json {
    #[serde(rename(deserialize = "b64_json"))]
    pub b64_json: String,
}
