// https://beta.openai.com/docs/api-reference/moderations/create

use reqwest::{header, StatusCode};
use serde::Deserialize;
use serde_json::json;

use crate::app::{errors::DefaultApiError, models::api_error::ApiError};

pub async fn check_prompt(
    prompt: &str,
    openai_api_key: &str,
    client: &reqwest::Client,
) -> Result<OpenAiModerationResponse, ApiError> {
    let mut headers = header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert(
        "Authorization",
        ["Bearer ", openai_api_key].concat().parse().unwrap(),
    );

    let url = "https://api.openai.com/v1/moderations";
    let result = client
        .post(url)
        .headers(headers)
        .json(&json!({ "input": prompt }))
        .send()
        .await;

    match result {
        Ok(res) => match res.text().await {
            Ok(text) => match serde_json::from_str(&text) {
                Ok(openai_moderation_response) => Ok(openai_moderation_response),
                Err(_) => {
                    tracing::error!(%text);
                    Err(DefaultApiError::InternalServerError.value())
                }
            },
            Err(e) => {
                tracing::error!(%e);
                Err(DefaultApiError::InternalServerError.value())
            }
        },
        Err(e) => {
            tracing::error!("check_prompt, {:?}", e);
            Err(ApiError {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                message: "Failed to check prompt.".to_string(),
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct OpenAiModerationResponse {
    pub id: String,
    pub model: String,
    pub results: Vec<OpenAiModerationResult>,
}

#[derive(Debug, Deserialize)]
pub struct OpenAiModerationResult {
    pub categories: serde_json::Value,
    pub category_scores: serde_json::Value,
    pub flagged: bool,
}
