use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Envy {
    pub app_env: String,
    pub frontend_url: String,
    pub port: Option<u16>,

    pub database_url: String,

    pub jwt_secret: String,

    pub openai_api_key: String,

    pub backblaze_key_id: String,
    pub backblaze_app_key: String,
    pub backblaze_bucket_id: String,

    pub fcm_api_key: String,

    pub mail_host: String,
    pub mail_user: String,
    pub mail_pass: String,
}
