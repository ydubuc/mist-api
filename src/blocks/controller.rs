use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    headers::{authorization::Bearer, Authorization},
    Json, TypedHeader,
};
use reqwest::StatusCode;
use validator::Validate;

use crate::{app::models::api_error::ApiError, auth::jwt::models::claims::Claims, AppState};

use super::{dtos::get_blocks_filter_dto::GetBlocksFilterDto, models::block::Block, service};

pub async fn block(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
) -> Result<(), ApiError> {
    match Claims::from_header(authorization, &state.envy.jwt_secret) {
        Ok(claims) => service::block(&id, &claims, &state.pool).await,
        Err(e) => Err(e),
    }
}

pub async fn get_blocks(
    State(state): State<Arc<AppState>>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    Query(dto): Query<GetBlocksFilterDto>,
) -> Result<Json<Vec<Block>>, ApiError> {
    match Claims::from_header(authorization, &state.envy.jwt_secret) {
        Ok(claims) => {
            if let Err(e) = dto.validate() {
                return Err(ApiError {
                    code: StatusCode::BAD_REQUEST,
                    message: e.to_string(),
                });
            }

            match service::get_blocks(&dto, &claims, &state.pool).await {
                Ok(blocks) => Ok(Json(blocks)),
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

pub async fn unblock(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
) -> Result<(), ApiError> {
    match Claims::from_header(authorization, &state.envy.jwt_secret) {
        Ok(claims) => service::unblock(&id, &claims, &state.pool).await,
        Err(e) => Err(e),
    }
}
