use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{app::util::time, auth::dtos::register_dto::RegisterDto};

pub static USER_SORTABLE_FIELDS: [&str; 2] = ["created_at", "updated_at"];

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
    pub password_hash: String,
    #[sqlx(try_from = "i64")]
    pub updated_at: u64,
    #[sqlx(try_from = "i64")]
    pub created_at: u64,
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
            password_hash: hash,
            updated_at: current_time,
            created_at: current_time,
        };
    }
}
