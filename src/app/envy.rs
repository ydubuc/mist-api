use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Envy {
    pub app_env: String,
    pub frontend_url: String,
    pub port: Option<u16>,

    pub database_url: String,

    pub jwt_secret: String,
    pub revenuecat_webhook_secret: String,

    pub openai_api_key: String,
    pub dream_api_key: String,
    pub stable_horde_api_key: String,
    pub mist_stability_api_key: String,
    pub labml_api_key: String,
    pub replicate_api_key: String,
    pub modal_webhook_secret: String,

    pub backblaze_key_id: String,
    pub backblaze_app_key: String,
    pub backblaze_bucket_id: String,

    pub fcm_api_key: String,

    pub mail_host: String,
    pub mail_user: String,
    pub mail_pass: String,
}
