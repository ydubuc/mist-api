use reqwest::StatusCode;
use sqlx::Postgres;

use crate::{
    app::models::api_error::ApiError,
    media::{dtos::generate_media_dto::GenerateMediaDto, enums::media_generator::MediaGenerator},
};

use super::dtos::edit_user_ink_dto::EditUserInkDto;

pub fn calculate_ink_cost(dto: &GenerateMediaDto, number_generated: Option<u8>) -> i64 {
    let base_ink = match dto.generator.as_ref() {
        MediaGenerator::MIST => 10.0,
        MediaGenerator::STABLE_HORDE => 10.0,
        MediaGenerator::DALLE => 40.0,
        _ => panic!(
            "calculate_ink_cost for generator {} not implemented.",
            dto.generator
        ),
    };

    let ink_per_pixel: f64 = base_ink / (512.0 * 512.0);

    let number = match number_generated {
        Some(number_generated) => match number_generated > dto.number {
            true => dto.number,
            false => number_generated,
        },
        None => dto.number,
    };

    let pixels = ((number as u64) * (dto.width as u64 * dto.height as u64)) as u64;

    let ink_cost = ((pixels as f64) * ink_per_pixel).round() as i64;

    return ink_cost;
}

pub async fn edit_user_ink_by_id(
    id: &str,
    dto: &EditUserInkDto,
    tx: &mut sqlx::Transaction<'_, Postgres>,
) -> Result<(), ApiError> {
    let sql_result = dto.to_sql();
    let Ok(sql) = sql_result else {
        return Err(sql_result.err().unwrap());
    };

    let mut sqlx = sqlx::query(&sql);

    if let Some(ink_increase) = dto.ink_increase {
        sqlx = sqlx.bind(ink_increase);
    }
    if let Some(ink_decrease) = dto.ink_decrease {
        sqlx = sqlx.bind(ink_decrease);
    }
    if let Some(ink_sum_increase) = dto.ink_sum_increase {
        sqlx = sqlx.bind(ink_sum_increase);
    }
    if let Some(ink_sum_decrease) = dto.ink_sum_decrease {
        sqlx = sqlx.bind(ink_sum_decrease);
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
                tracing::error!("update_user_ink_by_id ({}): NO ROWS AFFECTED", id);

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
