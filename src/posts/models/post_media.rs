use serde::{Deserialize, Serialize};

use crate::media::models::media::Media;

#[derive(Debug, Serialize, Deserialize)]
pub struct PostMedia {
    pub id: String,
    pub user_id: String,
    pub file_id: String,
    pub url: String,
    pub width: i16,
    pub height: i16,
    pub mime_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<String>,
    pub source: String,
    pub created_at: i64,
}

impl PostMedia {
    pub fn from_media(media: Media) -> Self {
        Self {
            id: media.id,
            user_id: media.user_id,
            file_id: media.file_id,
            url: media.url,
            width: media.width,
            height: media.height,
            mime_type: media.mime_type,
            seed: media.seed,
            source: media.source,
            created_at: media.created_at,
        }
    }
}
