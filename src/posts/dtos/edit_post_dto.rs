use std::time::{SystemTime, UNIX_EPOCH};

use axum::http::StatusCode;
use serde::Deserialize;
use validator::Validate;

use crate::{app::models::api_error::ApiError, auth::jwt::models::claims::Claims};

#[derive(Debug, Deserialize, Validate)]
pub struct EditPostDto {
    #[validate(length(
        min = 1,
        max = 512,
        message = "title must be between 1 and 512 characters."
    ))]
    pub title: Option<String>,
    #[validate(length(
        min = 1,
        max = 65535,
        message = "content must be between 1 and 65535 characters."
    ))]
    pub content: Option<String>,
}

impl EditPostDto {
    pub fn to_sql(&self, claims: &Claims) -> Result<String, ApiError> {
        let mut sql = "UPDATE posts SET ".to_string();
        let mut clauses = Vec::new();

        let mut index: u8 = 1;

        // SET CLAUSES
        if self.title.is_some() {
            clauses.push(["title = $", &index.to_string()].concat());
            index += 1;
        }
        if self.content.is_some() {
            clauses.push(["content = $", &index.to_string()].concat());
            index += 1;
        }

        // CLAUSES BUILDER
        if clauses.len() == 0 {
            return Err(ApiError {
                code: StatusCode::BAD_REQUEST,
                message: "Received nothing to edit.".to_string(),
            });
        }

        for (i, clause) in clauses.iter().enumerate() {
            if i != 0 {
                sql.push_str(", ");
            }

            sql.push_str(&clause);
        }

        let updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        sql.push_str(&[", updated_at = ", &updated_at.to_string()].concat());

        sql.push_str(&[" WHERE id = $", &index.to_string()].concat());
        sql.push_str(&[" AND user_id = '", &claims.id, "'"].concat());
        sql.push_str(" RETURNING *");

        println!("{}", sql);

        Ok(sql)
    }
}
