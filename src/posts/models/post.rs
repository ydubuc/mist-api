use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    app::util::time, auth::jwt::models::claims::Claims, media::models::media::Media,
    posts::dtos::create_post_dto::CreatePostDto,
};

use super::post_media::PostMedia;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Post {
    pub id: String,
    pub user_id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<sqlx::types::Json<PostMedia>>,
    pub updated_at: i64,
    pub created_at: i64,
}

impl Post {
    pub fn new(claims: &Claims, dto: &CreatePostDto, media: Option<Media>) -> Self {
        let current_time = time::current_time_in_secs() as i64;

        return Self {
            id: Uuid::new_v4().to_string(),
            user_id: claims.id.to_string(),
            title: dto.title.to_string(),
            content: dto.content.to_owned(),
            media: match media {
                Some(media) => Some(sqlx::types::Json(PostMedia::from_media(media))),
                None => None,
            },
            updated_at: current_time,
            created_at: current_time,
        };
    }

    pub fn sortable_fields() -> [&'static str; 2] {
        return ["created_at", "updated_at"];
    }
}
