use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BackblazeUploadUrlResponse {
    #[serde(rename(deserialize = "bucketId"))]
    pub bucket_id: String,
    #[serde(rename(deserialize = "uploadUrl"))]
    pub upload_url: String,
    #[serde(rename(deserialize = "authorizationToken"))]
    pub authorization_token: String,
}
