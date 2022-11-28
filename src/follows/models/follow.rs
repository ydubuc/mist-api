use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::{app::util::time, auth::jwt::models::claims::Claims};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Follow {
    pub id: String,
    pub user_id: String,
    pub follows_id: String,
    pub followed_at: i64,

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

impl Follow {
    pub fn new(claims: &Claims, follows_id: &str) -> Self {
        return Self {
            id: format!("{}{}", claims.id, follows_id),
            user_id: claims.id.to_string(),
            follows_id: follows_id.to_string(),
            followed_at: time::current_time_in_secs() as i64,

            user_username: None,
            user_displayname: None,
            user_avatar_url: None,
        };
    }

    pub fn sortable_fields() -> [&'static str; 1] {
        return ["followed_at"];
    }
}
