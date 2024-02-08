use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::{
    app::util::time,
    media::{dtos::generate_media_dto::GenerateMediaDto, models::media::Media},
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
    pub media: Option<sqlx::types::Json<Vec<PostMedia>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_media_dto: Option<sqlx::types::Json<GenerateMediaDto>>,
    pub published: bool,
    pub featured: bool,
    pub reports_count: i16,
    pub updated_at: i64,
    pub created_at: i64,

    // JOINED USER
    #[serde(skip_serializing_if = "Option::is_none")]
    #[sqlx(default)] // this is because the value does not exist on the posts table itself
    pub user_username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[sqlx(default)] // this is because the value does not exist on the posts table itself
    pub user_displayname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[sqlx(default)] // this is because the value does not exist on the posts table itself
    pub user_avatar_url: Option<String>,
}

impl Post {
    pub fn from_media(media: Vec<Media>) -> Option<Self> {
        let Some(first_media) = media.first().cloned() else {
            return None;
        };
        let Some(post_id) = first_media.post_id else {
            return None;
        };
        let Some(generate_media_dto) = &first_media.generate_media_dto else {
            return None;
        };

        let current_time = time::current_time_in_secs() as i64;

        let mut post_media = Vec::new();
        for m in media {
            post_media.push(PostMedia::from_media(m));
        }

        return Some(Self {
            id: post_id.to_string(),
            user_id: first_media.user_id,
            title: generate_media_dto.prompt.to_string(),
            content: None,
            media: Some(sqlx::types::Json(post_media)),
            generate_media_dto: Some(generate_media_dto.clone()),
            published: generate_media_dto.publish.unwrap_or(false),
            featured: false,
            reports_count: 0,
            updated_at: current_time,
            created_at: current_time,

            user_username: None,
            user_displayname: None,
            user_avatar_url: None,
        });
    }

    pub fn sortable_fields() -> [&'static str; 3] {
        return ["reports_count", "created_at", "updated_at"];
    }
}
