use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    headers::{authorization::Bearer, Authorization},
    http::StatusCode,
    Json, TypedHeader,
};
use validator::Validate;

use crate::{
    app::{models::api_error::ApiError, structs::json_from_request::JsonFromRequest},
    auth::jwt::models::claims::Claims,
    AppState,
};

use super::{
    dtos::{edit_post_dto::EditPostDto, get_posts_filter_dto::GetPostsFilterDto},
    models::post::Post,
    service,
};

// pub async fn create_post(
//     State(state): State<Arc<AppState>>,
//     TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
//     JsonFromRequest(dto): JsonFromRequest<CreatePostDto>,
// ) -> Result<Json<Post>, ApiError> {
//     match Claims::from_header(authorization, &state.envy.jwt_secret) {
//         Ok(claims) => {
//             if let Err(e) = dto.validate() {
//                 return Err(ApiError {
//                     code: StatusCode::BAD_REQUEST,
//                     message: e.to_string(),
//                 });
//             }

//             match service::create_post(&dto, &claims, &state.pool).await {
//                 Ok(post) => Ok(Json(post)),
//                 Err(e) => Err(e),
//             }
//         }
//         Err(e) => Err(e),
//     }
// }

pub async fn get_posts(
    State(state): State<Arc<AppState>>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    Query(dto): Query<GetPostsFilterDto>,
) -> Result<Json<Vec<Post>>, ApiError> {
    match Claims::from_header(authorization, &state.envy.jwt_secret) {
        Ok(claims) => {
            if let Err(e) = dto.validate() {
                return Err(ApiError {
                    code: StatusCode::BAD_REQUEST,
                    message: e.to_string(),
                });
            }

            match service::get_posts(&dto, &claims, &state.pool).await {
                Ok(posts) => Ok(Json(posts)),
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

pub async fn get_post_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    // TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
) -> Result<Json<Post>, ApiError> {
    match service::get_post_by_id(&id, &state.pool).await {
        Ok(post) => Ok(Json(post)),
        Err(e) => Err(e),
    }

    // match Claims::from_header(authorization, &state.envy.jwt_secret) {
    //     Ok(claims) => match service::get_post_by_id(&id, &claims, &state.pool).await {
    //         Ok(post) => Ok(Json(post)),
    //         Err(e) => Err(e),
    //     },
    //     Err(e) => Err(e),
    // }
}

pub async fn edit_post_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    JsonFromRequest(dto): JsonFromRequest<EditPostDto>,
) -> Result<Json<Post>, ApiError> {
    match Claims::from_header(authorization, &state.envy.jwt_secret) {
        Ok(claims) => {
            if let Err(e) = dto.validate() {
                return Err(ApiError {
                    code: StatusCode::BAD_REQUEST,
                    message: e.to_string(),
                });
            }

            match service::edit_post_by_id(&id, &dto, &claims, &state.pool).await {
                Ok(post) => Ok(Json(post)),
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

pub async fn report_post_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
) -> Result<(), ApiError> {
    match Claims::from_header(authorization, &state.envy.jwt_secret) {
        Ok(claims) => service::report_post_by_id(&id, &claims, &state.pool).await,
        Err(e) => Err(e),
    }
}

pub async fn delete_post_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
) -> Result<(), ApiError> {
    match Claims::from_header(authorization, &state.envy.jwt_secret) {
        Ok(claims) => return service::delete_post_by_id(&id, &claims, &state.pool).await,
        Err(e) => Err(e),
    }
}
