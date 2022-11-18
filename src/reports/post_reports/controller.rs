use axum::{
    extract::{Path, State},
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};

use crate::{app::models::api_error::ApiError, auth::jwt::models::claims::Claims, AppState};

use super::service;

pub async fn report_post_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
) -> Result<(), ApiError> {
    match Claims::from_header(authorization, &state.envy.jwt_secret) {
        Ok(claims) => service::report_post_by_id(&id, &claims, &state.pool).await,
        Err(e) => Err(e),
    }
}
