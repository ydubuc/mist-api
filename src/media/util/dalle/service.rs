use b2_backblaze::B2;
use serde_json::json;
use sqlx::PgPool;
use tracing::Level;
use uuid::Uuid;

extern crate reqwest;

use crate::{
    app::{
        self, env::Env, errors::DefaultApiError, models::api_error::ApiError,
        util::multipart::models::file_properties::FileProperties,
    },
    auth::jwt::models::claims::Claims,
    media::{
        self, dtos::generate_media_dto::GenerateMediaDto, models::media::Media, util::backblaze,
    },
};
use reqwest::{header, StatusCode};

use super::models::dalle_generate_image_response::DalleGenerateImageResponse;

pub async fn generate_media(
    dto: &GenerateMediaDto,
    claims: &Claims,
    pool: &PgPool,
    b2: &B2,
) -> Result<Vec<Media>, ApiError> {
    match dalle_generate_image(dto).await {
        Ok(dalle_response) => {
            let mut files_properties = Vec::new();

            for data in &dalle_response.data {
                match app::util::reqwest::get_bytes(&data.url).await {
                    Ok(bytes) => {
                        let uuid = Uuid::new_v4().to_string();
                        let file_properties = FileProperties {
                            id: uuid.to_string(),
                            field_name: uuid.to_string(),
                            file_name: uuid.to_string(),
                            mime_type: mime::IMAGE_PNG,
                            data: bytes,
                        };

                        files_properties.push(file_properties);
                    }
                    Err(_) => {
                        // failed to get bytes
                        // skip to next data
                    }
                }
            }

            let sub_folder = Some(["media/", &claims.id].concat());
            match backblaze::service::upload_files(files_properties, &sub_folder, b2).await {
                Ok(responses) => {
                    let media = media::service::create_media_from_responses(responses, claims, b2);

                    if media.len() == 0 {
                        return Err(ApiError {
                            code: StatusCode::INTERNAL_SERVER_ERROR,
                            message: "Failed to upload files.".to_string(),
                        });
                    }

                    match media::service::upload_media(media, pool).await {
                        Ok(m) => Ok(m),
                        Err(e) => Err(e),
                    }
                }
                Err(e) => Err(e),
            }
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
