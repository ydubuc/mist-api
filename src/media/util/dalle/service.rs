use serde_json::json;
use tracing::Level;

extern crate reqwest;

use crate::{
    app::{env::Env, errors::DefaultApiError, models::api_error::ApiError},
    auth::jwt::models::claims::Claims,
    media::{dtos::generate_media_dto::GenerateMediaDto, models::media::Media},
};
use reqwest::{header, StatusCode};

use super::models::dalle_generate_image_response::DalleGenerateImageResponse;

pub async fn generate_media(
    claims: &Claims,
    dto: &GenerateMediaDto,
) -> Result<Vec<Media>, ApiError> {
    match dalle_generate_image(dto).await {
        Ok(dalle_response) => {
            let media = Media::from_dalle(dto, &dalle_response, claims);
            Ok(media)
        }
        Err(e) => Err(e),
    }
}

async fn dalle_generate_image(
    dto: &GenerateMediaDto,
) -> Result<DalleGenerateImageResponse, ApiError> {
    let size = [
        dto.width.to_string(),
        "x".to_string(),
        dto.height.to_string(),
    ]
    .concat();

    let valid_sizes = [
        "256x256".to_string(),
        "512x512".to_string(),
        "1024x1024".to_string(),
    ];

    if !valid_sizes.contains(&size) {
        return Err(ApiError {
            code: StatusCode::BAD_REQUEST,
            message: ["Size must be one of: ", &valid_sizes.join(",")].concat(),
        });
    }

    let openai_api_key =
        std::env::var(Env::OPENAI_API_KEY).expect("environment: OPENAPI_API_KEY missing");

    let mut headers = header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert(
        "Authorization",
        ["Bearer ", &openai_api_key].concat().parse().unwrap(),
    );

    let client = reqwest::Client::new();
    let result = client
        .post("https://api.openai.com/v1/images/generations")
        .headers(headers)
        .body(
            json!({
                "prompt": dto.prompt,
                "n": dto.number,
                "size": size
            })
            .to_string(),
        )
        .send()
        .await;

    match result {
        Ok(res) => match res.text().await {
            Ok(text) => match serde_json::from_str(&text) {
                Ok(dalle_response) => Ok(dalle_response),
                Err(_) => {
                    tracing::event!(Level::ERROR, %text);
                    Err(DefaultApiError::InternalServerError.value())
                }
            },
            Err(e) => {
                tracing::event!(Level::ERROR, %e);
                Err(DefaultApiError::InternalServerError.value())
            }
        },
        Err(e) => {
            tracing::event!(Level::ERROR, %e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}
