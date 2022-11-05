use axum::{extract::rejection::JsonRejection, http::StatusCode, response::IntoResponse};
use serde_json::json;

#[derive(Debug)]
pub struct ApiError {
    pub code: StatusCode,
    pub message: String,
}

impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        let code = match rejection {
            JsonRejection::JsonDataError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            JsonRejection::JsonSyntaxError(_) => StatusCode::BAD_REQUEST,
            JsonRejection::MissingJsonContentType(_) => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        Self {
            code,
            message: rejection.to_string(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let payload = json!({
            "message": self.message,
        });

        (self.code, axum::Json(payload)).into_response()
    }
}
