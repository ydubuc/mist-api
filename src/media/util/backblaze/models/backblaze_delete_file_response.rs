use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BackblazeDeleteFileResponse {
    #[serde(rename(deserialize = "fileId"))]
    pub file_id: String,
    #[serde(rename(deserialize = "fileName"))]
    pub file_name: String,
}
