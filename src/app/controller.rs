use std::sync::Arc;

use axum::extract::State;

use crate::AppState;

use super::models::api_error::ApiError;

pub async fn get_root(State(_state): State<Arc<AppState>>) -> Result<(), ApiError> {
    Ok(())
}
