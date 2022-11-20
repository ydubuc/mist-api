use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

use crate::{
    app::util::time, auth::jwt::models::claims::Claims,
    generate_media_requests::enums::generate_media_request_status::GenerateMediaRequestStatus,
    media::dtos::generate_media_dto::GenerateMediaDto,
};

#[derive(Debug, Clone, Serialize, Deserialize, Validate, FromRow)]
pub struct GenerateMediaRequest {
    pub id: String,
    pub user_id: String,
    pub status: String,
    pub generate_media_dto: sqlx::types::Json<GenerateMediaDto>,
    pub created_at: i64,
}

impl GenerateMediaRequest {
    pub fn new(claims: &Claims, generate_media_dto: &GenerateMediaDto) -> Self {
        return Self {
            id: Uuid::new_v4().to_string(),
            user_id: claims.id.to_string(),
            status: GenerateMediaRequestStatus::Processing.value().to_string(),
            generate_media_dto: sqlx::types::Json(generate_media_dto.clone()),
            created_at: time::current_time_in_secs() as i64,
        };
    }

    pub fn sortable_fields() -> [&'static str; 1] {
        return ["created_at"];
    }
}
