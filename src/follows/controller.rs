use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    headers::{authorization::Bearer, Authorization},
    Json, TypedHeader,
};
use reqwest::StatusCode;
use validator::Validate;

use crate::{app::models::api_error::ApiError, auth::jwt::models::claims::Claims, AppState};

use super::{dtos::get_follows_filter_dto::GetFollowsFilterDto, models::follow::Follow, service};

pub async fn follow(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
) -> Result<(), ApiError> {
    match Claims::from_header(authorization, &state.envy.jwt_secret) {
        Ok(claims) => service::follow(&id, &claims, &state.pool).await,
        Err(e) => Err(e),
    }
}

pub async fn get_follows(
    State(state): State<Arc<AppState>>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    Query(dto): Query<GetFollowsFilterDto>,
) -> Result<Json<Vec<Follow>>, ApiError> {
    match Claims::from_header(authorization, &state.envy.jwt_secret) {
        Ok(claims) => {
            if let Err(e) = dto.validate() {
                return Err(ApiError {
                    code: StatusCode::BAD_REQUEST,
                    message: e.to_string(),
                });
            }

            match service::get_follows(&dto, &claims, &state.pool).await {
                Ok(follows) => Ok(Json(follows)),
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

pub async fn unfollow(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
) -> Result<(), ApiError> {
    match Claims::from_header(authorization, &state.envy.jwt_secret) {
        Ok(claims) => service::unfollow(&id, &claims, &state.pool).await,
        Err(e) => Err(e),
    }
}
