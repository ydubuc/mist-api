use mime::IMAGE_PNG;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    app::util::time,
    auth::jwt::models::claims::Claims,
    media::{
        dtos::generate_media_dto::GenerateMediaDto, enums::media_source::MediaSource,
        services::dalle::models::dalle_generate_image_response::DalleGenerateImageResponse,
    },
};

use super::import_media_response::ImportMediaResponse;

pub static MEDIA_SORTABLE_FIELDS: [&str; 1] = ["created_at"];

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Media {
    pub id: String,
    pub user_id: String,
    pub url: String,
    #[sqlx(try_from = "i16")]
    pub width: u16,
    #[sqlx(try_from = "i16")]
    pub height: u16,
    pub mime_type: String,
    pub source: String,
    #[sqlx(try_from = "i64")]
    pub created_at: u64,
}

impl Media {
    pub fn from_dalle(
        dto: &GenerateMediaDto,
        dalle_generate_image_response: &DalleGenerateImageResponse,
        claims: &Claims,
    ) -> Vec<Self> {
        let mut vec = Vec::new();

        for data in &dalle_generate_image_response.data {
            vec.push(Self {
                id: Uuid::new_v4().to_string(),
                user_id: claims.id.to_string(),
                url: data.url.to_string(),
                width: dto.width.to_owned(),
                height: dto.height.to_owned(),
                mime_type: IMAGE_PNG.to_string(),
                source: MediaSource::Dalle.value(),
                created_at: time::current_time_in_secs(),
            })
        }

        return vec;
    }

    pub fn from_import(import_media_response: &ImportMediaResponse, claims: &Claims) -> Self {
        Self {
            id: import_media_response.id.to_string(),
            user_id: claims.id.to_string(),
            url: import_media_response.download_url.to_string(),
            width: 512,
            height: 512,
            mime_type: import_media_response
                .backblaze_upload_file_response
                .content_type
                .to_string(),
            source: MediaSource::Import.value(),
            created_at: time::current_time_in_secs(),
        }
    }
}
