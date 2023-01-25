use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct ReceiveWebhookDto {
    pub request_id: String,
    pub images: Vec<String>,
}
