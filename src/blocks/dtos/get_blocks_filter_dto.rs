use reqwest::StatusCode;
use serde::Deserialize;
use validator::Validate;

use crate::{app::models::api_error::ApiError, blocks::models::block::Block};

#[derive(Debug, Deserialize, Validate)]
pub struct GetBlocksFilterDto {
    pub id: Option<String>,
    #[validate(length(equal = 36, message = "user_id must be 36 characters."))]
    pub user_id: Option<String>,
    #[validate(length(equal = 36, message = "blocked_id must be 36 characters."))]
    pub blocked_id: Option<String>,
    pub sort: Option<String>,
    pub cursor: Option<String>,
    #[validate(range(min = 1, max = 100, message = "limit must equal or less than 100."))]
    pub limit: Option<u8>,
}

impl GetBlocksFilterDto {
    pub fn to_sql(&self) -> Result<String, ApiError> {
        let mut sql = "SELECT blocks.*".to_string();

        // JOIN USER
        sql.push_str(", users.id as user_id, users.username as user_username, users.displayname as user_displayname FROM blocks LEFT JOIN users ON blocks.blocked_id = users.id");

        let mut clauses = Vec::new();

        let mut sort_field = "blocked_at".to_string();
        let mut sort_order = "DESC".to_string();
        let mut page_limit: u8 = 50;

        let mut index: u8 = 0;

        // WHERE CLAUSES
        if self.id.is_some() {
            index += 1;
            clauses.push(["id = $", &index.to_string()].concat());
        }
        if self.user_id.is_some() {
            index += 1;
            clauses.push(["user_id = $", &index.to_string()].concat());
        }
        if self.blocked_id.is_some() {
            index += 1;
            clauses.push(["blocked_id = $", &index.to_string()].concat());
        }

        // SORT
        if let Some(sort) = &self.sort {
            let sort_params: Vec<&str> = sort.split(",").collect();

            if sort_params.len() != 2 {
                return Err(ApiError {
                    code: StatusCode::BAD_REQUEST,
                    message: "Malformed sort query.".to_string(),
                });
            }
            if !Block::sortable_fields().contains(&sort_params[0]) {
                return Err(ApiError {
                    code: StatusCode::BAD_REQUEST,
                    message: "Invalid sort field.".to_string(),
                });
            }

            sort_field = sort_params[0].to_string();
            sort_order = sort_params[1].to_uppercase();

            let direction = match sort_order.as_str() {
                "ASC" => ">",
                "DESC" => "<",
                _ => {
                    return Err(ApiError {
                        code: StatusCode::BAD_REQUEST,
                        message: "Malformed sort query.".to_string(),
                    })
                }
            };

            if let Some(cursor) = &self.cursor {
                let cursor_params: Vec<&str> = cursor.split(",").collect();

                if cursor_params.len() != 2 {
                    return Err(ApiError {
                        code: StatusCode::BAD_REQUEST,
                        message: "Malformed cursor.".to_string(),
                    });
                }

                let cursor_value = cursor_params[0].to_string();
                let cursor_id = cursor_params[1].to_string();

                clauses.push(
                    [
                        "(blocks.",
                        &sort_field,
                        ", blocks.id) ",
                        direction,
                        " (",
                        &cursor_value,
                        ", '",
                        &cursor_id,
                        "')",
                    ]
                    .concat(),
                );
            }
        }

        // CLAUSES BUILDER
        let mut has_inserted_where = false;

        for clause in clauses {
            if !has_inserted_where {
                sql.push_str(" WHERE ");
                has_inserted_where = true;
            } else {
                sql.push_str(" AND ");
            }

            sql.push_str(&clause);
        }

        // ORDER BY
        sql.push_str(&[" ORDER BY blocks.", &sort_field, " ", &sort_order].concat());

        if self.cursor.is_some() {
            sql.push_str(&[", blocks.id ", &sort_order].concat());
        }

        // LIMIT
        if let Some(limit) = self.limit {
            page_limit = limit;
        }

        sql.push_str(&[" LIMIT ", &page_limit.to_string()].concat());

        tracing::debug!(sql);

        Ok(sql.to_string())
    }
}
