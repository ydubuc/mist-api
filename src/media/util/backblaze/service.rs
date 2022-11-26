use std::{sync::Arc, time::Instant};

use b2_backblaze::B2;
use reqwest::header;
use serde_json::json;
use tokio::sync::RwLock;

use crate::{
    app::{
        errors::DefaultApiError, models::api_error::ApiError,
        util::multipart::models::file_properties::FileProperties,
    },
    media::util::backblaze::enums::backblaze_delete_file_error_code::BackBlazeDeleteFileErrorCode,
};

use super::structs::{
    backblaze_delete_file_error::BackblazeDeleteFileError,
    backblaze_delete_file_response::BackblazeDeleteFileResponse,
    backblaze_upload_file_response::BackblazeUploadFileResponse,
    backblaze_upload_url_response::BackblazeUploadUrlResponse,
};

// pub async fn upload_files(
//     files_properties: &Vec<FileProperties>,
//     sub_folder: &Option<String>,
//     b2: &B2,
// ) -> Vec<Result<BackblazeUploadFileResponse, ApiError>> {
//     let mut futures = Vec::with_capacity(files_properties.len());

//     for file_properties in files_properties {
//         futures.push(upload_file(&file_properties, sub_folder, b2));
//     }

//     futures::future::join_all(futures).await
// }

pub async fn upload_file(
    file_properties: &FileProperties,
    sub_folder: &Option<String>,
    b2: &Arc<RwLock<B2>>,
) -> Result<BackblazeUploadFileResponse, ApiError> {
    let upload_url_result = get_upload_url(b2).await;

    match upload_url_result {
        Ok(upload_url_res) => {
            let path = match sub_folder {
                Some(folder) => [folder, "/", &file_properties.id].concat(),
                None => ["public/", &file_properties.id].concat(),
            };

            let mut headers = header::HeaderMap::new();
            headers.insert(
                "Authorization",
                upload_url_res.authorization_token.parse().unwrap(),
            );
            headers.insert("X-Bz-File-Name", path.parse().unwrap());
            headers.insert(
                "Content-Type",
                file_properties.mime_type.to_string().parse().unwrap(),
            );
            headers.insert(
                "X-Bz-Content-Sha1",
                "do_not_verify".to_string().parse().unwrap(),
            );
            headers.insert("X-Bz-Info-Author", "Mist".to_string().parse().unwrap());

            let client = reqwest::Client::new();
            let result = client
                .post(upload_url_res.upload_url)
                .headers(headers)
                .body(file_properties.data.clone())
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

async fn get_upload_url(b2: &Arc<RwLock<B2>>) -> Result<BackblazeUploadUrlResponse, ApiError> {
    check_token(b2).await;

    let b2 = b2.read().await;

    let mut headers = header::HeaderMap::new();
    headers.insert("Authorization", b2.authorizationToken.parse().unwrap());

    let url = b2.apiUrl.to_string() + "/b2api/v2/b2_get_upload_url";
    let client = reqwest::Client::new();
    let result = client
        .post(url)
        .headers(headers)
        .body(
            json!({
                "bucketId": b2.bucketId.to_string()
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

pub async fn delete_file(
    file_name: &str,
    file_id: &str,
    b2: &Arc<RwLock<B2>>,
) -> Result<Option<BackblazeDeleteFileResponse>, ApiError> {
    check_token(b2).await;

    let b2 = b2.read().await;

    let mut headers = header::HeaderMap::new();
    headers.insert("Authorization", b2.authorizationToken.parse().unwrap());

    let client = reqwest::Client::new();
    let result = client
        .post([&b2.apiUrl, "/b2api/v2/b2_delete_file_version"].concat())
        .headers(headers)
        .body(
            json!({
                "fileName": file_name,
                "fileId": file_id
            })
            .to_string(),
        )
        .send()
        .await;

    match result {
        Ok(res) => match res.text().await {
            Ok(text) => match serde_json::from_str(&text) {
                Ok(delete_file_res) => Ok(Some(delete_file_res)),
                Err(_) => {
                    let delete_file_error_result: Result<
                        BackblazeDeleteFileError,
                        serde_json::Error,
                    > = serde_json::from_str(&text);

                    if let Ok(delete_file_error) = delete_file_error_result {
                        match delete_file_error.code.as_ref() {
                            BackBlazeDeleteFileErrorCode::FILE_NOT_PRESENT => return Ok(None),
                            _ => {
                                tracing::error!(%text);
                                Err(DefaultApiError::InternalServerError.value())
                            }
                        }
                    } else {
                        tracing::error!(%text);
                        Err(DefaultApiError::InternalServerError.value())
                    }
                }
            },
            Err(e) => {
                tracing::error!(%e);
                Err(DefaultApiError::InternalServerError.value())
            }
        },
        Err(e) => {
            tracing::error!(%e);
            return Err(DefaultApiError::InternalServerError.value());
        }
    }
}

async fn check_token(b2: &Arc<RwLock<B2>>) {
    let _b2 = b2.read().await;

    if _b2.token_time.elapsed().as_secs() > 43200 {
        drop(_b2);

        let mut b2 = b2.write().await;

        match b2.check_token().await {
            Ok(_) => tracing::info!("updated b2 token"),
            Err(e) => tracing::error!(%e),
        }
    }
}
