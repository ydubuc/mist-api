use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BackblazeDeleteFileError {
    #[serde(rename(deserialize = "code"))]
    pub code: String,
    #[serde(rename(deserialize = "message"))]
    pub message: String,
    #[serde(rename(deserialize = "status"))]
    status: u16,
}
