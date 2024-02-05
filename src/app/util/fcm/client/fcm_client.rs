use std::{
    collections::HashMap,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::{header, Body, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::config::Config;

#[derive(Clone, Debug)]
pub struct FcmClient {
    pub config: Config,
    pub http_client: reqwest::Client,
    pub authorization_token: String,
    pub token_time: Instant,
}

impl FcmClient {
    pub fn new(config: Config, http_client: Option<reqwest::Client>) -> FcmClient {
        FcmClient {
            config,
            http_client: http_client.unwrap_or(
                reqwest::ClientBuilder::new()
                    .pool_max_idle_per_host(std::usize::MAX)
                    .build()
                    .unwrap(),
            ),
            authorization_token: String::new(),
            token_time: Instant::now(),
        }
    }

    pub async fn login(&mut self) -> Result<(), &'static str> {
        let Ok(system_time) = SystemTime::now().duration_since(UNIX_EPOCH) else {
            return Err("time went backwards");
        };
        let current_time = system_time.as_secs();

        let claims = serde_json::json!({
            "iss": self.config.client_email,
            "scope": "https://www.googleapis.com/auth/firebase.messaging",
            "aud": "https://www.googleapis.com/oauth2/v4/token",
            "exp": current_time + 3600,
            "iat": current_time,
        });

        let Ok(encoding_key) = EncodingKey::from_rsa_pem(self.config.private_key.as_bytes()) else {
            return Err("unable to encode private key");
        };

        let encode_result = encode(&Header::new(Algorithm::RS256), &claims, &encoding_key);

        if let Err(e) = encode_result {
            tracing::error!(%e);
            return Err("failed to encode token");
        }

        let token = encode_result.unwrap();

        let mut headers = header::HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());

        let mut body = HashMap::new();
        body.insert("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer");
        body.insert("assertion", &token);

        let result = self
            .http_client
            .post("https://www.googleapis.com/oauth2/v4/token")
            .headers(headers)
            .json(&body)
            .send()
            .await;

        match result {
            Ok(res) => match res.text().await {
                Ok(text) => {
                    let fcm_oauth_response_result: Result<FcmOAuthResponse, serde_json::Error> =
                        serde_json::from_str(&text);

                    match fcm_oauth_response_result {
                        Ok(fcm_oauth_response) => {
                            self.authorization_token = fcm_oauth_response.access_token;
                            self.token_time = Instant::now();

                            return Ok(());
                        }
                        Err(_) => {
                            tracing::error!(%text);
                            return Err("failed to login to fcm");
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(%e);
                    return Err("failed to login to fcm");
                }
            },
            Err(e) => {
                tracing::error!(%e);
                return Err("failed to login to fcm");
            }
        }
    }

    pub async fn check_token(&mut self) -> Result<(), &'static str> {
        let has_expired = self.token_time.elapsed().as_secs() > 3600;

        if !has_expired {
            return Ok(());
        } else {
            return self.login().await;
        }
    }

    pub async fn send(&self, message: FcmMessage) -> Result<(), String> {
        let fcm_message = json!({
            "message": {
                "token": message.token,
                "notification": {
                    "title": message.title,
                    "body": message.body,
                    "click_action": match message.click_action {
                        Some(click_action) => click_action,
                        None => "none".to_string(),
                    },
                    "sound": "default"
                }
            }
        });
        let Ok(payload) = serde_json::to_vec(&fcm_message) else {
            return Err(message.token);
        };

        let mut headers = header::HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers.insert("Content-Length", (payload.len() as u64).into());
        headers.insert(
            "Authorization",
            ["Bearer ", &self.authorization_token]
                .concat()
                .parse()
                .unwrap(),
        );

        let result = self
            .http_client
            .post(format!(
                "https://fcm.googleapis.com/v1/projects/{}/messages:send",
                self.config.project_name
            ))
            .headers(headers)
            .body(Body::from(payload))
            .send()
            .await;

        match result {
            Ok(res) => match res.status() {
                StatusCode::OK => return Ok(()),
                _ => {
                    tracing::error!("{:?}", res.text().await);
                    return Err(message.token);
                }
            },
            Err(e) => {
                tracing::error!(%e);
                return Err(message.token);
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FcmMessage {
    pub token: String,
    pub title: String,
    pub body: String,
    pub click_action: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FcmOAuthResponse {
    #[serde(rename(deserialize = "access_token"))]
    pub access_token: String,
}
