use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PostMedia {
    #[sqlx(rename = "media_url")]
    pub url: String,
    #[sqlx(rename = "media_width")]
    #[sqlx(try_from = "i16")]
    pub width: u16,
    #[sqlx(rename = "media_height")]
    #[sqlx(try_from = "i16")]
    pub height: u16,
    #[sqlx(rename = "media_mime_type")]
    pub mime_type: String,
    #[sqlx(rename = "media_source")]
    pub source: String,
}
