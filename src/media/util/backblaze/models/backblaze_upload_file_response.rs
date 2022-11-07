use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BackblazeUploadFileResponse {
    #[serde(rename(deserialize = "bucketId"))]
    pub bucket_id: String,
    #[serde(rename(deserialize = "contentLength"))]
    pub content_length: u64,
    #[serde(rename(deserialize = "contentMd5"))]
    pub content_md5: String,
    #[serde(rename(deserialize = "contentSha1"))]
    pub content_sha1: String,
    #[serde(rename(deserialize = "contentType"))]
    pub content_type: String,
    #[serde(rename(deserialize = "fileId"))]
    pub file_id: String,
    #[serde(rename(deserialize = "fileName"))]
    pub file_name: String,
    #[serde(rename(deserialize = "uploadTimestamp"))]
    pub upload_timestamp: u64,
}
