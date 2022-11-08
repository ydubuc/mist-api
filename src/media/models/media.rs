use serde::{Deserialize, Serialize};
use sqlx::FromRow;

pub static MEDIA_SORTABLE_FIELDS: [&str; 1] = ["created_at"];

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Media {
    pub id: String,
    pub user_id: String,
    pub file_id: String,
    pub url: String,
    pub width: i16,
    pub height: i16,
    pub mime_type: String,
    pub source: String,
    pub created_at: i64,
}
