use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::{
    app::util::time, auth::jwt::models::claims::Claims, media::enums::media_source::MediaSource,
};

use super::import_media_response::ImportMediaResponse;

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

impl Media {
    pub fn from_import(import_media_response: &ImportMediaResponse, claims: &Claims) -> Self {
        Self {
            id: import_media_response.id.to_string(),
            user_id: claims.id.to_string(),
            file_id: import_media_response
                .backblaze_upload_file_response
                .file_id
                .to_string(),
            url: import_media_response.download_url.to_string(),
            width: import_media_response.size.width as i16,
            height: import_media_response.size.height as i16,
            mime_type: import_media_response
                .backblaze_upload_file_response
                .content_type
                .to_string(),
            source: MediaSource::Import.value(),
            created_at: time::current_time_in_secs() as i64,
        }
    }
}
