use std::sync::Arc;

use axum::{
    extract::{Multipart, State},
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use axum_macros::debug_handler;
use reqwest::StatusCode;

use crate::{
    app::{
        errors::DefaultApiError, models::api_error::ApiError,
        structs::json_from_request::JsonFromRequest,
        util::multipart::multipart::get_files_properties,
    },
    generate_media_requests, media,
    webhooks::modal::dtos::receive_webhook_dto::ReceiveWebhookDto,
    AppState,
};

#[debug_handler]
pub async fn receive_webhook(
    State(state): State<Arc<AppState>>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    multipart: Multipart,
) -> Result<(), ApiError> {
    if authorization.0.token() != state.envy.modal_webhook_secret {
        return Err(DefaultApiError::PermissionDenied.value());
    }

    // let id = &dto.request_id;
    let files_properties = get_files_properties(multipart).await;

    if files_properties.len() == 0 {
        println!("received nothing");

        return Err(ApiError {
            code: StatusCode::BAD_REQUEST,
            message: "Received nothing to upload.".to_string(),
        });
    }

    for file_properties in &files_properties {
        tracing::debug!("{:?}", file_properties.field_name)
        // if !file_properties
        //     .mime_type
        //     .starts_with(&mime::IMAGE.to_string())
        // {
        //     return Err(ApiError {
        //         code: StatusCode::BAD_REQUEST,
        //         message: "Files must be images.".to_string(),
        //     });
        // }
    }

    // match generate_media_requests::service::get_generate_media_request_by_id_as_admin(
    //     id,
    //     &state.pool,
    // )
    // .await
    // {
    //     Ok(request) => media::apis::modal::service::on_receive_webhook(request, dto, state),
    //     Err(e) => {
    //         if e.code == StatusCode::NOT_FOUND {
    //             tracing::error!("receive_webhook failed: request not found");
    //             return Ok(());
    //         } else {
    //             return Err(DefaultApiError::InternalServerError.value());
    //         }
    //     }
    // }

    return Ok(());
}

// #[debug_handler]
// pub async fn receive_webhook(
//     State(state): State<Arc<AppState>>,
//     TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
//     JsonFromRequest(dto): JsonFromRequest<ReceiveWebhookDto>,
// ) -> Result<(), ApiError> {
//     if authorization.0.token() != state.envy.modal_webhook_secret {
//         return Err(DefaultApiError::PermissionDenied.value());
//     }

//     let id = &dto.request_id;

//     match generate_media_requests::service::get_generate_media_request_by_id_as_admin(
//         id,
//         &state.pool,
//     )
//     .await
//     {
//         Ok(request) => media::apis::modal::service::on_receive_webhook(request, dto, state),
//         Err(e) => {
//             if e.code == StatusCode::NOT_FOUND {
//                 tracing::error!("receive_webhook failed: request not found");
//                 return Ok(());
//             } else {
//                 return Err(DefaultApiError::InternalServerError.value());
//             }
//         }
//     }

//     return Ok(());
// }
