use mime::IMAGE_PNG;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    app::util::time,
    auth::jwt::models::claims::Claims,
    media::{
        constants::Source, dtos::create_media_dto::CreateMediaDto, services::dalle::DalleResponse,
    },
};

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
    pub fn new(claims: &Claims, dto: &CreateMediaDto, dalle_response: &DalleResponse) -> Vec<Self> {
        let mut vec = Vec::new();
        let mut width: u16 = 512;
        let mut height: u16 = 512;

        let size_split: Vec<&str> = dto.size.split("x").collect();
        if size_split.len() == 2 {
            width = size_split[0].parse().unwrap_or(width);
            height = size_split[1].parse().unwrap_or(height);
        }

        for data in &dalle_response.data {
            vec.push(Self {
                id: Uuid::new_v4().to_string(),
                user_id: claims.id.to_string(),
                url: data.url.to_string(),
                width: width,
                height: height,
                mime_type: IMAGE_PNG.to_string(),
                source: Source::DALLE.to_string(),
                created_at: time::current_time_in_secs(),
            })
        }

        return vec;
    }
}
