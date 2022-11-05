use axum::{
    extract::{Path, Query, State},
    headers::{authorization::Bearer, Authorization},
    http::StatusCode,
    Json, TypedHeader,
};
use validator::Validate;

use crate::{
    app::models::{api_error::ApiError, json_from_request::JsonFromRequest},
    auth::jwt::models::claims::Claims,
    AppState,
};

use super::{
    dtos::{
        create_post_dto::CreatePostDto, edit_post_dto::EditPostDto,
        get_posts_filter_dto::GetPostsFilterDto,
    },
    models::post::Post,
    service,
};

pub async fn create_post(
    State(state): State<AppState>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    JsonFromRequest(dto): JsonFromRequest<CreatePostDto>,
) -> Result<Json<Post>, ApiError> {
    match Claims::from_header(authorization) {
        Ok(claims) => match dto.validate() {
            Ok(_) => match service::create_post(&claims, &dto, &state.pool).await {
                Ok(post) => Ok(Json(post)),
                Err(e) => Err(e),
            },
            Err(e) => Err(ApiError {
                code: StatusCode::BAD_REQUEST,
                message: e.to_string(),
            }),
        },
        Err(e) => Err(e),
    }
}

pub async fn get_posts(
    State(state): State<AppState>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    Query(dto): Query<GetPostsFilterDto>,
) -> Result<Json<Vec<Post>>, ApiError> {
    match Claims::from_header(authorization) {
        Ok(_) => match dto.validate() {
            Ok(_) => match service::get_posts(&dto, &state.pool).await {
                Ok(posts) => Ok(Json(posts)),
                Err(e) => Err(e),
            },
            Err(e) => Err(ApiError {
                code: StatusCode::BAD_REQUEST,
                message: e.to_string(),
            }),
        },
        Err(e) => Err(e),
    }
}

pub async fn get_post_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
) -> Result<Json<Post>, ApiError> {
    match Claims::from_header(authorization) {
        Ok(_) => match service::get_post_by_id(&id, &state.pool).await {
            Ok(post) => Ok(Json(post)),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

pub async fn edit_post_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    JsonFromRequest(dto): JsonFromRequest<EditPostDto>,
) -> Result<Json<Post>, ApiError> {
    match Claims::from_header(authorization) {
        Ok(claims) => match dto.validate() {
            Ok(_) => match service::edit_post_by_id(&claims, &id, &dto, &state.pool).await {
                Ok(post) => Ok(Json(post)),
                Err(e) => Err(e),
            },
            Err(e) => Err(ApiError {
                code: StatusCode::BAD_REQUEST,
                message: e.to_string(),
            }),
        },
        Err(e) => Err(e),
    }
}

pub async fn delete_post_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
) -> Result<(), ApiError> {
    match Claims::from_header(authorization) {
        Ok(claims) => return service::delete_post_by_id(&claims, &id, &state.pool).await,
        Err(e) => Err(e),
    }
}
