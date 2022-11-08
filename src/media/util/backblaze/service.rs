use b2_backblaze::B2;
use reqwest::header;
use serde_json::json;

use crate::{
    app::{
        errors::DefaultApiError, models::api_error::ApiError,
        util::multipart::models::file_properties::FileProperties,
    },
    auth::jwt::models::claims::Claims,
    media::models::{import_media_response::ImportMediaResponse, media::Media},
};

use super::models::{
    backblaze_delete_file_response::BackblazeDeleteFileResponse,
    backblaze_upload_file_response::BackblazeUploadFileResponse,
    backblaze_upload_url_response::BackblazeUploadUrlResponse,
};

pub async fn upload_files(
    files_properties: Vec<FileProperties>,
    sub_folder: &Option<String>,
    b2: &B2,
) -> Result<Vec<(String, BackblazeUploadFileResponse)>, ApiError> {
    let client = reqwest::Client::new();
    let mut responses = Vec::new();

    for file_properties in files_properties {
        match upload_file(file_properties, sub_folder, &client, b2).await {
            Ok(res) => responses.push(res),
            Err(e) => tracing::error!(%e.message),
        }
    }

    Ok(responses)
}

async fn upload_file(
    file_properties: FileProperties,
    sub_folder: &Option<String>,
    client: &reqwest::Client,
    b2: &B2,
) -> Result<(String, BackblazeUploadFileResponse), ApiError> {
    let upload_url_result = get_upload_url(&b2.bucketId, b2, client).await;

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
                        Ok(upload_file_res) => Ok((file_properties.id, upload_file_res)),
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

pub fn create_media_from_responses(
    responses: Vec<(String, BackblazeUploadFileResponse)>,
    claims: &Claims,
    b2: &B2,
) -> Vec<Media> {
    let mut vec = Vec::new();

    for res in responses {
        let download_url = [
            &b2.downloadUrl,
            "/b2api/v1/b2_download_file_by_id?fileId=",
            &res.1.file_id,
        ]
        .concat();

        let import_media_res = ImportMediaResponse {
            id: res.0.to_string(),
            download_url,
            backblaze_upload_file_response: res.1,
        };

        vec.push(Media::from_import(&import_media_res, claims));
    }

    return vec;
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

pub async fn delete_file(
    file_name: &str,
    file_id: &str,
    b2: &B2,
) -> Result<BackblazeDeleteFileResponse, ApiError> {
    let client = reqwest::Client::new();

    let mut headers = header::HeaderMap::new();
    headers.insert("Authorization", b2.authorizationToken.parse().unwrap());

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
                Ok(delete_file_res) => Ok(delete_file_res),
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
            return Err(DefaultApiError::InternalServerError.value());
        }
    }
}
