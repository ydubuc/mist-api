use axum::http::StatusCode;
use sqlx::PgPool;

use crate::{
    app::{
        errors::DefaultApiError,
        models::api_error::ApiError,
        util::sqlx::{get_code_from_db_err, SqlStateCodes},
    },
    auth::jwt::models::claims::Claims,
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
    claims: &Claims,
    dto: &CreatePostDto,
    pool: &PgPool,
) -> Result<Post, ApiError> {
    let post = Post::new(claims, dto, &None);

    let sqlx_result = sqlx::query(
        "
        INSERT INTO posts (
            id, user_id, title, content, updated_at, created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        ",
    )
    .bind(&post.id)
    .bind(&post.user_id)
    .bind(&post.title)
    .bind(&post.content)
    .bind(post.updated_at.to_owned() as i64)
    .bind(post.created_at.to_owned() as i64)
    .execute(pool)
    .await;

    if let Some(error) = sqlx_result.as_ref().err() {
        println!("{}", error);
    }

    match sqlx_result {
        Ok(_) => Ok(post),
        Err(e) => {
            let Some(db_err) = e.as_database_error() else {
                return Err(DefaultApiError::InternalServerError.value());
            };

            let Some(code) = get_code_from_db_err(db_err) else {
                return Err(DefaultApiError::InternalServerError.value());
            };

            match code.as_str() {
                SqlStateCodes::UNIQUE_VIOLATION => Err(ApiError {
                    code: StatusCode::CONFLICT,
                    message: "Post already exists.".to_string(),
                }),
                _ => Err(DefaultApiError::InternalServerError.value()),
            }
        }
    }
}

pub async fn get_posts(dto: &GetPostsFilterDto, pool: &PgPool) -> Result<Vec<Post>, ApiError> {
    let sql_result = dto.to_sql();
    let Ok(sql) = sql_result else {
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

    let sqlx_result = sqlx.fetch_all(pool).await;

    if let Some(error) = sqlx_result.as_ref().err() {
        println!("{}", error);
    }

    match sqlx_result {
        Ok(posts) => Ok(posts),
        Err(_) => Err(DefaultApiError::InternalServerError.value()),
    }
}

pub async fn get_post_by_id(id: &str, pool: &PgPool) -> Result<Post, ApiError> {
    let sqlx_result = sqlx::query_as::<_, Post>(
        "
        SELECT * FROM posts WHERE id = $1
        ",
    )
    .bind(id)
    .fetch_optional(pool)
    .await;

    if let Some(error) = sqlx_result.as_ref().err() {
        println!("{}", error);
    }

    match sqlx_result {
        Ok(post) => match post {
            Some(post) => Ok(post),
            None => Err(PostsApiError::PostNotFound.value()),
        },
        Err(_) => Err(DefaultApiError::InternalServerError.value()),
    }
}

pub async fn edit_post_by_id(
    claims: &Claims,
    id: &str,
    dto: &EditPostDto,
    pool: &PgPool,
) -> Result<Post, ApiError> {
    let sql_result = dto.to_sql(claims);
    let Ok(sql) = sql_result else {
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

    let sqlx_result = sqlx.fetch_optional(pool).await;

    if let Some(error) = sqlx_result.as_ref().err() {
        println!("{}", error);
    }

    match sqlx_result {
        Ok(post) => match post {
            Some(post) => Ok(post),
            None => Err(PostsApiError::PostNotFound.value()),
        },
        Err(_) => Err(DefaultApiError::InternalServerError.value()),
    }
}

pub async fn delete_post_by_id(claims: &Claims, id: &str, pool: &PgPool) -> Result<(), ApiError> {
    let sqlx_result = sqlx::query(
        "
        DELETE FROM posts WHERE id = $1 AND user_id = $2
        ",
    )
    .bind(id)
    .bind(&claims.id)
    .execute(pool)
    .await;

    if let Some(error) = sqlx_result.as_ref().err() {
        println!("{}", error);
    }

    match sqlx_result {
        Ok(result) => match result.rows_affected() > 0 {
            true => Ok(()),
            false => Err(PostsApiError::PostNotFound.value()),
        },
        Err(_) => Err(DefaultApiError::InternalServerError.value()),
    }
}
