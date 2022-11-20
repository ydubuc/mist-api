use axum::http::StatusCode;
use serde::Deserialize;

use crate::{
    app::{models::api_error::ApiError, util::time},
    auth::jwt::models::claims::Claims,
};

#[derive(Debug, Deserialize)]
pub struct EditDeviceDto {
    pub messaging_token: Option<String>,
}

impl EditDeviceDto {
    pub fn to_sql(&self, claims: &Claims) -> Result<String, ApiError> {
        let mut sql = "UPDATE devices SET ".to_string();
        let mut clauses = Vec::new();

        let mut index: u8 = 1;

        // SET CLAUSES
        if self.messaging_token.is_some() {
            clauses.push(["messaging_token = $", &index.to_string()].concat());
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

        let updated_at = time::current_time_in_secs();
        sql.push_str(&[", updated_at = ", &updated_at.to_string()].concat());

        sql.push_str(&[" WHERE id = $", &index.to_string()].concat());
        sql.push_str(&[" AND user_id = '", &claims.id, "'"].concat());
        sql.push_str(" RETURNING *");

        println!("{}", sql);

        Ok(sql)
    }
}
