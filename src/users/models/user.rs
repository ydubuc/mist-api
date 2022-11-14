use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{app::util::time, auth::dtos::register_dto::RegisterDto};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub username_key: String,
    pub displayname: String,
    #[serde(skip_serializing)]
    pub email: String,
    #[serde(skip_serializing)]
    pub email_key: String,
    #[serde(skip_serializing)]
    pub email_pending: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub updated_at: i64,
    pub created_at: i64,
}

impl User {
    pub fn new(dto: &RegisterDto, hash: String) -> Self {
        let current_time = time::current_time_in_secs();

        return Self {
            id: Uuid::new_v4().to_string(),
            username: dto.username.to_string(),
            username_key: dto.username.to_lowercase(),
            displayname: dto.username.to_string(),
            email: dto.email.to_string(),
            email_key: dto.email.to_lowercase(),
            email_pending: None,
            avatar_url: None,
            password_hash: hash,
            updated_at: current_time as i64,
            created_at: current_time as i64,
        };
    }

    pub fn nullable_fields() -> [&'static str; 1] {
        return ["avatar_url"];
    }

    pub fn sortable_fields() -> [&'static str; 2] {
        return ["created_at", "updated_at"];
    }
}
