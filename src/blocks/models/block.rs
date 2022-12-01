use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::{app::util::time, auth::jwt::models::claims::Claims};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Block {
    pub id: String,
    pub user_id: String,
    pub blocked_id: String,
    pub blocked_at: i64,

    // JOINED USER
    #[serde(skip_serializing_if = "Option::is_none")]
    #[sqlx(default)] // this is because the value does not exist on the posts table itself
    pub user_username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[sqlx(default)] // this is because the value does not exist on the posts table itself
    pub user_displayname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[sqlx(default)] // this is because the value does not exist on the posts table itself
    pub user_avatar_url: Option<String>,
}

impl Block {
    pub fn new(claims: &Claims, blocked_id: &str) -> Self {
        return Self {
            id: format!("{}{}", claims.id, blocked_id),
            user_id: claims.id.to_string(),
            blocked_id: blocked_id.to_string(),
            blocked_at: time::current_time_in_secs() as i64,

            user_username: None,
            user_displayname: None,
            user_avatar_url: None,
        };
    }

    pub fn sortable_fields() -> [&'static str; 1] {
        return ["blocked_at"];
    }
}
