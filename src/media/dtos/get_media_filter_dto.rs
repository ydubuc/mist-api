use axum::http::StatusCode;
use serde::Deserialize;
use validator::Validate;

use crate::{app::models::api_error::ApiError, media::models::media::MEDIA_SORTABLE_FIELDS};

#[derive(Debug, Deserialize, Validate)]
pub struct GetMediaFilterDto {
    #[validate(length(equal = 36, message = "id must be 36 characters."))]
    pub id: Option<String>,
    #[validate(length(equal = 36, message = "id must be 36 characters."))]
    pub user_id: Option<String>,
    #[validate(url)]
    pub url: Option<String>,
    pub mime_type: Option<String>,
    pub sort: Option<String>,
    pub cursor: Option<String>,
    #[validate(range(max = 100, message = "limit must be equal or less than 100."))]
    pub limit: Option<u8>,
}

impl GetMediaFilterDto {
    pub fn to_sql(&self) -> Result<String, ApiError> {
        let mut sql = "SELECT * FROM media".to_string();
        let mut clauses = Vec::new();

        let mut sort_field = "created_at".to_string();
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
        if self.url.is_some() {
            index += 1;
            clauses.push(["url = $", &index.to_string()].concat());
        }
        if self.mime_type.is_some() {
            index += 1;
            clauses.push(["mime_type = $", &index.to_string()].concat());
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
            if !MEDIA_SORTABLE_FIELDS.contains(&sort_params[0]) {
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
                clauses.push([&sort_field, " ", direction, " ", cursor].concat());
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
        sql.push_str(&[" ORDER BY ", &sort_field, " ", &sort_order].concat());

        // LIMIT
        if let Some(limit) = self.limit {
            page_limit = limit;
        }

        sql.push_str(&[" LIMIT ", &page_limit.to_string()].concat());

        println!("{:?}", sql);

        Ok(sql.to_string())
    }
}
