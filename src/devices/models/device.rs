use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{app::util::time, users::models::user::User};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Device {
    pub id: String,
    pub user_id: String,
    #[serde(skip_serializing)]
    pub refresh_token: String,
    pub updated_at: i64,
    pub created_at: i64,
}

impl Device {
    pub fn new(user: &User) -> Self {
        let current_time = time::current_time_in_secs() as i64;

        return Self {
            id: Uuid::new_v4().to_string(),
            user_id: user.id.to_string(),
            refresh_token: Uuid::new_v4().to_string(),
            updated_at: current_time,
            created_at: current_time,
        };
    }

    pub fn sortable_fields() -> [&'static str; 2] {
        return ["updated_at", "created_at"];
    }
}
