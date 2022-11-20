use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MistStabilityGenerateImagesResponse {
    #[serde(rename(deserialize = "base64Data"))]
    pub base64_data: Vec<String>,
}
