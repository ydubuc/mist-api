use axum::http::StatusCode;
use sqlx::PgPool;

use crate::{
    app::{
        errors::DefaultApiError,
        models::api_error::ApiError,
        util::sqlx::{get_code_from_db_err, SqlStateCodes},
    },
    auth::jwt::models::claims::Claims,
    media::{self, dtos::generate_media_dto::GenerateMediaDto, models::media::Media},
};

use super::{
    dtos::{
        create_post_dto::CreatePostDto, edit_post_dto::EditPostDto,
        get_posts_filter_dto::GetPostsFilterDto,
    },
    errors::PostsApiError,
    models::post::Post,
};

pub async fn create_post(
    dto: &CreatePostDto,
    claims: &Claims,
    pool: &PgPool,
) -> Result<Post, ApiError> {
    let mut media: Option<Vec<Media>> = None;

    if let Some(media_ids) = &dto.media_ids {
        let mut temp_media = Vec::new();

        for media_id in media_ids {
            match media::service::get_media_by_id(media_id, claims, pool).await {
                Ok(m) => {
                    if claims.id != m.user_id {
                        return Err(ApiError {
                            code: StatusCode::UNAUTHORIZED,
                            message: "Permission denied.".to_string(),
                        });
                    }

                    temp_media.push(m);
                }
                Err(e) => return Err(e),
            }
        }

        if temp_media.len() > 0 {
            media = Some(temp_media);
        }
    }

    let post = Post::new(claims, dto, media);

    let sqlx_result = sqlx::query(
        "
        INSERT INTO posts (
            id, user_id, title, content, media, updated_at, created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ",
    )
    .bind(&post.id)
    .bind(&post.user_id)
    .bind(&post.title)
    .bind(&post.content)
    .bind(&post.media)
    .bind(&post.updated_at)
    .bind(&post.created_at)
    .execute(pool)
    .await;

    match sqlx_result {
        Ok(_) => Ok(post),
        Err(e) => {
            let Some(db_err) = e.as_database_error()
            else {
                tracing::error!(%e);
                return Err(DefaultApiError::InternalServerError.value());
            };

            let Some(code) = get_code_from_db_err(db_err)
            else {
                tracing::error!(%e);
                return Err(DefaultApiError::InternalServerError.value());
            };

            match code.as_str() {
                SqlStateCodes::UNIQUE_VIOLATION => Err(ApiError {
                    code: StatusCode::CONFLICT,
                    message: "Post already exists.".to_string(),
                }),
                _ => {
                    tracing::error!(%e);
                    Err(DefaultApiError::InternalServerError.value())
                }
            }
        }
    }
}

pub async fn create_post_with_media(
    generate_media_dto: &GenerateMediaDto,
    media: &Vec<Media>,
    claims: &Claims,
    pool: &PgPool,
) -> Result<Post, ApiError> {
    let mut media_ids = Vec::new();

    for m in media {
        media_ids.push(m.id.to_string());
    }

    let dto = CreatePostDto {
        title: generate_media_dto.prompt.to_string(),
        content: None,
        media_ids: Some(media_ids),
    };

    match create_post(&dto, claims, pool).await {
        Ok(post) => Ok(post),
        Err(e) => Err(e),
    }
}

pub async fn get_posts(
    dto: &GetPostsFilterDto,
    claims: &Claims,
    pool: &PgPool,
) -> Result<Vec<Post>, ApiError> {
    let sql_result = dto.to_sql();
    let Ok(sql) = sql_result
    else {
        return Err(sql_result.err().unwrap());
    };

    let mut sqlx = sqlx::query_as::<_, Post>(&sql);

    if let Some(id) = &dto.id {
        sqlx = sqlx.bind(id);
    }
    if let Some(user_id) = &dto.user_id {
        sqlx = sqlx.bind(user_id)
    }
    if let Some(search) = &dto.search {
        sqlx = sqlx.bind(["%", search, "%"].concat())
    }

    match sqlx.fetch_all(pool).await {
        Ok(posts) => Ok(posts),
        Err(e) => {
            tracing::error!(%e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}

pub async fn get_post_by_id(id: &str, claims: &Claims, pool: &PgPool) -> Result<Post, ApiError> {
    let sqlx_result = sqlx::query_as::<_, Post>(
        r#"
        SELECT posts.*,
        users.id as user_id,
        users.username as user_username,
        users.displayname as user_displayname,
        users.avatar_url as user_avatar_url
        FROM posts
        LEFT JOIN users
        ON posts.user_id = users.id
        WHERE posts.id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await;

    // WORKING CODE BELOW

    // let sqlx_result = sqlx::query_as::<_, Post>(
    //     "
    //     SELECT * FROM posts
    //     WHERE posts.id = $1
    //     ",
    // )
    // .bind(id)
    // .bind(&claims.id)
    // .fetch_optional(pool)
    // .await;

    match sqlx_result {
        Ok(post) => match post {
            Some(post) => Ok(post),
            None => Err(PostsApiError::PostNotFound.value()),
        },
        Err(e) => {
            tracing::error!(%e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}

pub async fn get_post_by_id_as_admin(id: &str, pool: &PgPool) -> Result<Post, ApiError> {
    let sqlx_result = sqlx::query_as::<_, Post>(
        "
        SELECT * FROM posts
        WHERE posts.id = $1
        ",
    )
    .bind(id)
    .fetch_optional(pool)
    .await;

    match sqlx_result {
        Ok(post) => match post {
            Some(post) => Ok(post),
            None => Err(PostsApiError::PostNotFound.value()),
        },
        Err(e) => {
            tracing::error!(%e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}

pub async fn edit_post_by_id(
    id: &str,
    dto: &EditPostDto,
    claims: &Claims,
    pool: &PgPool,
) -> Result<Post, ApiError> {
    let sql_result = dto.to_sql(claims);
    let Ok(sql) = sql_result
    else {
        return Err(sql_result.err().unwrap());
    };

    let mut sqlx = sqlx::query_as::<_, Post>(&sql);

    if let Some(title) = &dto.title {
        sqlx = sqlx.bind(title);
    }
    if let Some(content) = &dto.content {
        sqlx = sqlx.bind(content);
    }
    sqlx = sqlx.bind(id);

    match sqlx.fetch_optional(pool).await {
        Ok(post) => match post {
            Some(post) => Ok(post),
            None => Err(PostsApiError::PostNotFound.value()),
        },
        Err(e) => {
            tracing::error!(%e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}

pub async fn delete_post_by_id(id: &str, claims: &Claims, pool: &PgPool) -> Result<(), ApiError> {
    let sqlx_result = sqlx::query(
        "
        DELETE FROM posts
        WHERE id = $1 AND user_id = $2
        ",
    )
    .bind(id)
    .bind(&claims.id)
    .execute(pool)
    .await;

    match sqlx_result {
        Ok(result) => match result.rows_affected() > 0 {
            true => Ok(()),
            false => Err(PostsApiError::PostNotFound.value()),
        },
        Err(e) => {
            tracing::error!(%e);
            Err(DefaultApiError::InternalServerError.value())
        }
    }
}
