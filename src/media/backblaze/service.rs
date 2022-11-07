use axum::{extract::Multipart, http::StatusCode};
use b2_backblaze::B2;
use reqwest::header;
use serde_json::json;
use uuid::Uuid;

use crate::app::{
    errors::DefaultApiError,
    models::api_error::ApiError,
    util::multipart::{models::file_properties::FileProperties, multipart::get_files_properties},
};

use super::models::{
    backblaze_upload_file_response::BackblazeUploadFileResponse,
    backblaze_upload_url_response::BackblazeUploadUrlResponse,
};

pub async fn upload_files(
    multipart: Multipart,
    b2: &B2,
) -> Result<Vec<BackblazeUploadFileResponse>, ApiError> {
    let files_properties = get_files_properties(multipart).await;

    if files_properties.len() == 0 {
        return Err(ApiError {
            code: StatusCode::BAD_REQUEST,
            message: "Found no files to upload.".to_string(),
        });
    }

    let client = reqwest::Client::new();
    let mut responses = Vec::new();

    for file_properties in files_properties {
        match upload_file(file_properties, &client, b2).await {
            Ok(res) => responses.push(res),
            Err(e) => tracing::error!(%e.message),
        }
    }

    Ok(responses)
}

async fn upload_file(
    file_properties: FileProperties,
    client: &reqwest::Client,
    b2: &B2,
) -> Result<BackblazeUploadFileResponse, ApiError> {
    let upload_url_result = get_upload_url(&b2.bucketId, b2, client).await;

    match upload_url_result {
        Ok(upload_url_res) => {
            let mut headers = header::HeaderMap::new();
            headers.insert(
                "Authorization",
                upload_url_res.authorization_token.parse().unwrap(),
            );
            headers.insert(
                "X-Bz-File-Name",
                Uuid::new_v4().to_string().parse().unwrap(),
            );
            headers.insert(
                "Content-Type",
                file_properties.mime_type.to_string().parse().unwrap(),
            );
            headers.insert(
                "X-Bz-Content-Sha1",
                "do_not_verify".to_string().parse().unwrap(),
            );
            headers.insert("X-Bz-Info-Author", "unknown".to_string().parse().unwrap());

            let result = client
                .post(upload_url_res.upload_url)
                .headers(headers)
                .body(file_properties.data)
                .send()
                .await;

            match result {
                Ok(res) => match res.text().await {
                    Ok(text) => match serde_json::from_str(&text) {
                        Ok(upload_file_res) => Ok(upload_file_res),
                        Err(_) => {
                            tracing::error!(%text);
                            return Err(DefaultApiError::InternalServerError.value());
                        }
                    },
                    Err(e) => {
                        tracing::error!(%e);
                        return Err(DefaultApiError::InternalServerError.value());
                    }
                },
                Err(e) => {
                    tracing::error!(%e);
                    return Err(DefaultApiError::InternalServerError.value());
                }
            }
        }
        Err(e) => Err(e),
    }
}

async fn get_upload_url(
    bucket_id: &str,
    b2: &B2,
    client: &reqwest::Client,
) -> Result<BackblazeUploadUrlResponse, ApiError> {
    let mut headers = header::HeaderMap::new();
    headers.insert("Authorization", b2.authorizationToken.parse().unwrap());

    let url = b2.apiUrl.to_string() + "/b2api/v2/b2_get_upload_url";
    let result = client
        .post(url)
        .headers(headers)
        .body(
            json!({
                "bucketId": bucket_id.to_string()
            })
            .to_string(),
        )
        .send()
        .await;

    match result {
        Ok(res) => match res.text().await {
            Ok(text) => match serde_json::from_str(&text) {
                Ok(upload_url_res) => Ok(upload_url_res),
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
            tracing::error!(%e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}
