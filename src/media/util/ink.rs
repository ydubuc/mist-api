use reqwest::StatusCode;
use sqlx::Postgres;

use crate::{app::models::api_error::ApiError, media::dtos::generate_media_dto::GenerateMediaDto};

pub fn calculate_ink_cost(dto: &GenerateMediaDto, number_generated: Option<u8>) -> i64 {
    let ink_per_pixel: f64 = 10.0 / (512.0 * 512.0);
    println!("ink per pixel {}", ink_per_pixel);

    let number = match number_generated {
        Some(number_generated) => number_generated,
        None => dto.number,
    };

    let pixels = ((number as u32) * (dto.width as u32 * dto.height as u32)) as u64;
    println!("pixels {}", pixels);

    let ink_cost = ((pixels as f64) * ink_per_pixel).round() as i64;
    println!("ink cost {}", ink_cost);

    return ink_cost;
}

pub async fn edit_user_ink_by_id(
    id: &str,
    dto: &EditUserInkDto,
    tx: &mut sqlx::Transaction<'_, Postgres>,
) -> Result<(), ApiError> {
    println!("update_user_ink_by_id");

    let sql_result = dto.to_sql();
    let Ok(sql) = sql_result
    else {
        return Err(sql_result.err().unwrap());
    };

    let mut sqlx = sqlx::query(&sql);

    if let Some(ink_increase) = dto.ink_increase {
        sqlx = sqlx.bind(ink_increase);
    }
    if let Some(ink_decrease) = dto.ink_decrease {
        sqlx = sqlx.bind(ink_decrease);
    }
    if let Some(ink_pending_increase) = dto.ink_pending_increase {
        sqlx = sqlx.bind(ink_pending_increase);
    }
    if let Some(ink_pending_decrease) = dto.ink_pending_decrease {
        sqlx = sqlx.bind(ink_pending_decrease);
    }

    sqlx = sqlx.bind(id);

    match sqlx.execute(&mut *tx).await {
        Ok(result) => match result.rows_affected() > 0 {
            true => Ok(()),
            false => {
                tracing::error!("<update_user_ink_by_id>: NO ROWS AFFECTED");
                tracing::error!(%id);

                return Err(ApiError {
                    code: StatusCode::NOT_FOUND,
                    message: "User not found.".to_string(),
                });
            }
        },
        Err(e) => {
            tracing::error!(%e);
            return Err(ApiError {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                message: "Failed to update user ink.".to_string(),
            });
        }
    }
}

pub struct EditUserInkDto {
    pub ink_increase: Option<i64>,
    pub ink_decrease: Option<i64>,
    pub ink_pending_increase: Option<i64>,
    pub ink_pending_decrease: Option<i64>,
}

impl EditUserInkDto {
    pub fn to_sql(&self) -> Result<String, ApiError> {
        let mut sql = "UPDATE users SET ".to_string();
        let mut clauses = Vec::new();

        let mut index: u8 = 1;

        if self.ink_increase.is_some() {
            clauses.push(["ink = ink + $", &index.to_string()].concat());
            index += 1;
        }
        if self.ink_decrease.is_some() {
            clauses.push(["ink = ink - $", &index.to_string()].concat());
            index += 1;
        }
        if self.ink_pending_increase.is_some() {
            clauses.push(["ink_pending = ink_pending + $", &index.to_string()].concat());
            index += 1;
        }
        if self.ink_decrease.is_some() {
            clauses.push(["ink_pending = ink_pending - $", &index.to_string()].concat());
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

        sql.push_str(&[" WHERE id = $", &index.to_string()].concat());

        println!("{}", sql);

        Ok(sql)
    }
}
