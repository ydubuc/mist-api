use std::time::{SystemTime, UNIX_EPOCH};

use axum::http::StatusCode;
use serde::Deserialize;
use validator::Validate;

use crate::app::models::api_error::ApiError;

#[derive(Debug, Deserialize, Validate)]
pub struct EditUserDto {
    #[validate(length(
        min = 3,
        max = 24,
        message = "displayname must be between 3 and 24 characters."
    ))]
    pub displayname: Option<String>,
}

impl EditUserDto {
    pub fn to_sql(&self) -> Result<String, ApiError> {
        let mut sql = "UPDATE users SET ".to_string();
        let mut clauses = Vec::new();

        let mut index: u8 = 1;

        // SET CLAUSES
        if self.displayname.is_some() {
            clauses.push(["displayname = $", &index.to_string()].concat());
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
        sql.push_str(" RETURNING *");

        println!("{}", sql);

        Ok(sql)
    }
}
