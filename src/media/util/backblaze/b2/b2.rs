use crate::Config;
use reqwest::header;
use serde::Deserialize;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct B2 {
    pub token_time: Instant,
    pub config: Config,
    pub account_id: String,
    pub authorization_token: String,
    pub api_url: String,
    pub download_url: String,
    pub bucket_id: String,
}

impl B2 {
    pub fn new(config: Config) -> B2 {
        B2 {
            token_time: Instant::now(),
            config,
            account_id: String::new(),
            authorization_token: String::new(),
            api_url: String::new(),
            download_url: String::new(),
            bucket_id: String::new(),
        }
    }

    pub fn set_bucket_id(&mut self, v: String) {
        self.bucket_id = v;
    }

    pub async fn login(&mut self) -> Result<(), &'static str> {
        return login(self).await;
    }

    pub async fn check_token(&mut self) -> Result<(), &'static str> {
        let has_expired = self.token_time.elapsed().as_secs() > 43200;

        if !has_expired {
            return Ok(());
        } else {
            return self.login().await;
        }
    }
}

async fn login(b2: &mut B2) -> Result<(), &'static str> {
    let authorization_token = base64::encode(format!("{}:{}", b2.config.id, b2.config.key));

    let mut headers = header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert(
        "Authorization",
        ["Basic ", &authorization_token].concat().parse().unwrap(),
    );

    let client = reqwest::Client::new();
    let url = format!("https://api.backblazeb2.com/b2api/v2/b2_authorize_account");

    let result = client.get(url).headers(headers).send().await;

    match result {
        Ok(res) => match res.text().await {
            Ok(text) => {
                let b2_authorize_response_result: Result<
                    B2AuthorizeAccountResponse,
                    serde_json::Error,
                > = serde_json::from_str(&text);

                match b2_authorize_response_result {
                    Ok(b2_authorize_response) => {
                        b2.account_id = b2_authorize_response.account_id;
                        b2.authorization_token = b2_authorize_response.authorization_token;
                        b2.api_url = b2_authorize_response.api_url;
                        b2.download_url = b2_authorize_response.download_url;
                        b2.token_time = Instant::now();

                        return Ok(());
                    }
                    Err(_) => {
                        tracing::error!(%text);
                        return Err("failed to login to b2");
                    }
                }
            }
            Err(e) => {
                tracing::error!(%e);
                return Err("failed to login to b2");
            }
        },
        Err(e) => {
            tracing::error!(%e);
            return Err("failed to login to b2");
        }
    }
}

#[derive(Debug, Deserialize)]
struct B2AuthorizeAccountResponse {
    #[serde(rename(deserialize = "accountId"))]
    pub account_id: String,
    #[serde(rename(deserialize = "authorizationToken"))]
    pub authorization_token: String,
    #[serde(rename(deserialize = "apiUrl"))]
    pub api_url: String,
    #[serde(rename(deserialize = "downloadUrl"))]
    pub download_url: String,
}
