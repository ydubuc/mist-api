use std::time::{SystemTime, UNIX_EPOCH};

use axum::http::StatusCode;
use serde::Deserialize;
use validator::Validate;

use crate::{app::models::api_error::ApiError, users::models::user::User};

#[derive(Debug, Deserialize, Validate)]
pub struct EditUserDto {
    #[validate(length(
        min = 3,
        max = 24,
        message = "username must be between 3 and 24 characters."
    ))]
    #[validate(regex(path = "crate::auth::dtos::USERNAME_REGEX"))]
    pub username: Option<String>,
    #[validate(length(
        min = 3,
        max = 24,
        message = "displayname must be between 3 and 24 characters."
    ))]
    #[validate(regex(path = "super::DISPLAYNAME_REGEX"))]
    pub displayname: Option<String>,
    pub avatar_media_id: Option<String>,
    pub nullify: Option<Vec<String>>,
}

impl EditUserDto {
    pub fn to_sql(&self) -> Result<String, ApiError> {
        let mut sql = "UPDATE users SET ".to_string();
        let mut clauses = Vec::new();

        let mut index: u8 = 1;

        // SET CLAUSES
        if self.username.is_some() {
            clauses.push(["username = $", &index.to_string()].concat());
            index += 1;

            clauses.push(["username_key = $", &index.to_string()].concat());
            index += 1;
        }
        if self.displayname.is_some() {
            clauses.push(["displayname = $", &index.to_string()].concat());
            index += 1;
        }
        if self.avatar_media_id.is_some() {
            clauses.push(["avatar_url = $", &index.to_string()].concat());
            index += 1;
        }
        if let Some(nullable_fields) = &self.nullify {
            for nullable_field in nullable_fields {
                if !User::nullable_fields().contains(&nullable_field.as_str()) {
                    return Err(ApiError {
                        code: StatusCode::BAD_REQUEST,
                        message: "One or more property cannot be nullified".to_string(),
                    });
                } else {
                    clauses.push([&nullable_field, " = NULL"].concat());
                }
            }
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
        sql.push_str(" RETURNING *");

        tracing::debug!(sql);

        Ok(sql)
    }
}
