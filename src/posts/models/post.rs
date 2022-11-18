use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    app::util::time,
    auth::jwt::models::claims::Claims,
    media::{dtos::generate_media_dto::GenerateMediaDto, models::media::Media},
    posts::dtos::create_post_dto::CreatePostDto,
};

use super::post_media::PostMedia;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Post {
    pub id: String,
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[sqlx(default)] // this is because the value does not exist on the posts table itself
    pub user_username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[sqlx(default)] // this is because the value does not exist on the posts table itself
    pub user_displayname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[sqlx(default)] // this is because the value does not exist on the posts table itself
    pub user_avatar_url: Option<String>,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<sqlx::types::Json<Vec<PostMedia>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_media_dto: Option<sqlx::types::Json<GenerateMediaDto>>,
    pub reports_count: i16,
    pub updated_at: i64,
    pub created_at: i64,
}

impl Post {
    pub fn new(claims: &Claims, dto: &CreatePostDto, media: Option<Vec<Media>>) -> Self {
        let current_time = time::current_time_in_secs() as i64;

        let post_media: Option<sqlx::types::Json<Vec<PostMedia>>>;
        let generate_media_dto: Option<sqlx::types::Json<GenerateMediaDto>>;

        if let Some(media) = media {
            if media.len() > 0 {
                match &media.first().unwrap().generate_media_dto {
                    Some(dto) => {
                        generate_media_dto = Some(dto.clone());
                    }
                    None => generate_media_dto = None,
                }

                let mut vec = Vec::new();

                for m in media {
                    vec.push(PostMedia::from_media(m));
                }

                post_media = Some(sqlx::types::Json(vec));
            } else {
                post_media = None;
                generate_media_dto = None;
            }
        } else {
            post_media = None;
            generate_media_dto = None;
        }

        return Self {
            id: Uuid::new_v4().to_string(),
            user_id: claims.id.to_string(),
            user_username: None,
            user_displayname: None,
            user_avatar_url: None,
            title: dto.title.to_string(),
            content: dto.content.to_owned(),
            media: post_media,
            generate_media_dto,
            reports_count: 0,
            updated_at: current_time,
            created_at: current_time,
        };
    }

    pub fn sortable_fields() -> [&'static str; 2] {
        return ["created_at", "updated_at"];
    }
}
