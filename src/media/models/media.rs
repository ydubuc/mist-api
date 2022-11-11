use b2_backblaze::B2;
use imagesize::ImageSize;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::{
    app::util::{multipart::models::file_properties::FileProperties, time},
    auth::jwt::models::claims::Claims,
    media::{
        enums::media_source::MediaSource,
        util::backblaze::models::backblaze_upload_file_response::BackblazeUploadFileResponse,
    },
};

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
    pub fn from_backblaze_responses(
        responses: Vec<(FileProperties, BackblazeUploadFileResponse)>,
        source: MediaSource,
        claims: &Claims,
        b2: &B2,
    ) -> Vec<Media> {
        let mut vec = Vec::new();

        for res in responses {
            let download_url = [
                &b2.downloadUrl,
                "/b2api/v1/b2_download_file_by_id?fileId=",
                &res.1.file_id,
            ]
            .concat();

            let size = match imagesize::blob_size(&res.0.data) {
                Ok(size) => size,
                Err(e) => ImageSize {
                    width: 512,
                    height: 512,
                },
            };

            let media = Media {
                id: res.0.id.to_string(),
                user_id: claims.id.to_string(),
                file_id: res.1.file_id.to_string(),
                url: download_url,
                width: size.width as i16,
                height: size.height as i16,
                mime_type: res.0.mime_type.to_string(),
                source: source.value(),
                created_at: time::current_time_in_secs() as i64,
            };

            vec.push(media);
        }

        return vec;
    }
}
