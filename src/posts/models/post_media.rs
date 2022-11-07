use serde::{Deserialize, Serialize};

use crate::media::models::media::Media;

#[derive(Debug, Serialize, Deserialize)]
pub struct PostMedia {
    pub url: String,
    pub width: i16,
    pub height: i16,
    pub mime_type: String,
    pub source: String,
}

impl PostMedia {
    pub fn from_media(media: Media) -> Self {
        Self {
            url: media.url.to_string(),
            width: media.width as i16,
            height: media.height as i16,
            mime_type: media.mime_type.to_string(),
            source: media.source.to_string(),
        }
    }
}
