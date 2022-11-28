use reqwest::StatusCode;
use serde::Deserialize;
use validator::Validate;

use crate::{app::models::api_error::ApiError, follows::models::follow::Follow};

#[derive(Debug, Deserialize, Validate)]
pub struct GetFollowsFilterDto {
    pub id: Option<String>,
    #[validate(length(equal = 36, message = "user_id must be 36 characters."))]
    pub user_id: Option<String>,
    #[validate(length(equal = 36, message = "follows_id must be 36 characters."))]
    pub follows_id: Option<String>,
    pub sort: Option<String>,
    pub cursor: Option<String>,
    #[validate(range(min = 1, max = 100, message = "limit must equal or less than 100."))]
    pub limit: Option<u8>,
}

impl GetFollowsFilterDto {
    pub fn to_sql(&self) -> Result<String, ApiError> {
        let mut sql = "SELECT follows.*".to_string();

        // TODO: if getting followers, I need to load user on user_id not follows_id
        // JOIN USER
        sql.push_str(", users.id as user_id, users.username as user_username, users.displayname as user_displayname, users.avatar_url as user_avatar_url FROM follows LEFT JOIN users ON follows.follows_id = users.id");

        let mut clauses = Vec::new();

        let mut sort_field = "followed_at".to_string();
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
        if self.follows_id.is_some() {
            index += 1;
            clauses.push(["follows_id = $", &index.to_string()].concat());
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
            if !Follow::sortable_fields().contains(&sort_params[0]) {
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
                        "(follows.",
                        &sort_field,
                        ", follows.id) ",
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

            // if let Some(cursor) = &self.cursor {
            //     clauses.push(["posts.", &sort_field, " ", direction, " ", cursor].concat());
            // }
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
        sql.push_str(&[" ORDER BY follows.", &sort_field, " ", &sort_order].concat());

        if self.cursor.is_some() {
            sql.push_str(&[", follows.id ", &sort_order].concat());
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
