use serde::Deserialize;
use serde_json::json;

extern crate reqwest;

use crate::{
    app::{env::Env, models::api_error::ApiError},
    auth::jwt::models::claims::Claims,
    media::{dtos::create_media_dto::CreateMediaDto, models::media::Media},
};
use reqwest::{header, StatusCode};

#[derive(Debug, Deserialize)]
pub struct DalleResponse {
    pub created: u64,
    pub data: Vec<DalleData>,
}

#[derive(Debug, Deserialize)]
pub struct DalleData {
    pub url: String,
}

pub async fn create_media(claims: &Claims, dto: &CreateMediaDto) -> Result<Vec<Media>, ApiError> {
    match dalle_generate_image(dto).await {
        Ok(dalle_response) => {
            let media = Media::new(claims, dto, &dalle_response);
            Ok(media)
        }
        Err(e) => Err(e),
    }
}

async fn dalle_generate_image(dto: &CreateMediaDto) -> Result<DalleResponse, ApiError> {
    let openai_api_key =
        std::env::var(Env::OPENAI_API_KEY).expect("environment: OPENAPI_API_KEY missing");

    let mut headers = header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert(
        "Authorization",
        ["Bearer ", &openai_api_key].concat().parse().unwrap(),
    );

    let client = reqwest::Client::new();
    let res = client
        .post("https://api.openai.com/v1/images/generations")
        .headers(headers)
        .body(
            json!({
                "prompt": dto.prompt,
                "n": dto.number,
                "size": "512x512"
            })
            .to_string(),
        )
        .send()
        .await;

    match res {
        Ok(res) => match res.text().await {
            Ok(text) => match serde_json::from_str(&text) {
                Ok(dalle_response) => Ok(dalle_response),
                Err(_) => Err(ApiError {
                    code: StatusCode::INTERNAL_SERVER_ERROR,
                    message: "Failed to deserialize response.".to_string(),
                }),
            },
            Err(e) => Err(ApiError {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                message: e.to_string(),
            }),
        },
        Err(e) => Err(ApiError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            message: e.to_string(),
        }),
    }
}
