use serde::Deserialize;

use crate::media::apis::dream::models::input_spec::InputSpec;

#[derive(Debug, Deserialize)]
pub struct DreamTaskResponse {
    #[serde(rename(deserialize = "id"))]
    pub id: String,
    #[serde(rename(deserialize = "input_spec"))]
    pub input_spec: Option<InputSpec>,
    #[serde(rename(deserialize = "state"))]
    pub state: String,
    #[serde(rename(deserialize = "photo_url_list"))]
    pub photo_url_list: Option<Vec<String>>,
    #[serde(rename(deserialize = "result"))]
    pub result: Option<String>,
    #[serde(rename(deserialize = "use_target_image"))]
    pub use_target_image: bool,
    #[serde(rename(deserialize = "target_image_url"))]
    pub target_image_url: Option<TargetImageUrl>,
    #[serde(rename(deserialize = "updated_at"))]
    pub updated_at: String,
    #[serde(rename(deserialize = "created_at"))]
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct TargetImageUrl {
    #[serde(rename(deserialize = "url"))]
    pub url: String,
    #[serde(rename(deserialize = "fields"))]
    pub fields: Fields,
}

#[derive(Debug, Deserialize)]
pub struct Fields {
    #[serde(rename(deserialize = "key"))]
    pub key: String,
    #[serde(rename(deserialize = "AwsAccessKeyId"))]
    pub aws_access_key_id: String,
    #[serde(rename(deserialize = "policy"))]
    pub policy: String,
    #[serde(rename(deserialize = "ksignatureey"))]
    pub signature: String,
}
