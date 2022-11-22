use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MistStabilityGenerateImagesResponse {
    #[serde(rename(deserialize = "data"))]
    pub data: Vec<MistStabilityImageData>,
}

#[derive(Debug, Deserialize)]
pub struct MistStabilityImageData {
    #[serde(rename(deserialize = "base64"))]
    pub base64: String,
    #[serde(rename(deserialize = "seed"))]
    pub seed: String,
}
