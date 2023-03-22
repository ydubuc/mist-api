use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct ReceiveWebhookDto {
    pub request_id: String,
    pub output: Vec<ReceiveWebhookDtoOutput>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ReceiveWebhookDtoOutput {
    pub seed: String,
    pub url: String,
}
