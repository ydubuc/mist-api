use b2_backblaze::B2;
use imagesize::ImageSize;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::{
    app::util::time,
    auth::jwt::models::claims::Claims,
    media::{
        dtos::generate_media_dto::GenerateMediaDto, enums::media_source::MediaSource,
        util::backblaze::structs::backblaze_upload_file_response::BackblazeUploadFileResponse,
    },
};

pub static MEDIA_SORTABLE_FIELDS: [&str; 1] = ["created_at"];

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Media {
    pub id: String,
    pub user_id: String,
    pub file_id: String,
    pub url: String,
    pub width: i16,
    pub height: i16,
    pub mime_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_media_dto: Option<sqlx::types::Json<GenerateMediaDto>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<String>,
    pub source: String,
    pub created_at: i64,
}

impl Media {
    pub fn from_dto(
        dto: &GenerateMediaDto,
        seed: Option<&str>,
        b2_upload_responses: &BackblazeUploadFileResponse,
        claims: &Claims,
        b2: &B2,
    ) -> Media {
        let download_url = [
            &b2.downloadUrl,
            "/b2api/v1/b2_download_file_by_id?fileId=",
            &b2_upload_responses.file_id,
        ]
        .concat();

        return Media {
            id: b2_upload_responses.file_id.to_string(),
            user_id: claims.id.to_string(),
            file_id: b2_upload_responses.file_id.to_string(),
            url: download_url,
            width: dto.width as i16,
            height: dto.height as i16,
            mime_type: b2_upload_responses.content_type.to_string(),
            generate_media_dto: Some(sqlx::types::Json(dto.clone())),
            seed: match seed {
                Some(seed) => Some(seed.to_string()),
                None => None,
            },
            source: dto.generator.to_string(),
            created_at: time::current_time_in_secs() as i64,
        };
    }

    pub fn from_import(
        b2_upload_responses: &BackblazeUploadFileResponse,
        image_size: &ImageSize,
        claims: &Claims,
        b2: &B2,
    ) -> Media {
        let download_url = [
            &b2.downloadUrl,
            "/b2api/v1/b2_download_file_by_id?fileId=",
            &b2_upload_responses.file_id,
        ]
        .concat();

        return Media {
            id: b2_upload_responses.file_id.to_string(),
            user_id: claims.id.to_string(),
            file_id: b2_upload_responses.file_id.to_string(),
            url: download_url,
            width: image_size.width.to_owned() as i16,
            height: image_size.height.to_owned() as i16,
            mime_type: b2_upload_responses.content_type.to_string(),
            generate_media_dto: None,
            seed: None,
            source: MediaSource::Import.value(),
            created_at: time::current_time_in_secs() as i64,
        };
    }
}
