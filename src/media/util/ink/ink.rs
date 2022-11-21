use reqwest::StatusCode;
use sqlx::Postgres;

use crate::{
    app::models::api_error::ApiError,
    media::{dtos::generate_media_dto::GenerateMediaDto, enums::media_generator::MediaGenerator},
};

use super::dtos::edit_user_dto::EditUserInkDto;

pub fn calculate_ink_cost(dto: &GenerateMediaDto, number_generated: Option<u8>) -> i64 {
    println!("calculating ink cost from dto number: {}", dto.number);
    println!("actual number generated: {:?}", number_generated);

    let base_ink = match dto.generator.as_ref() {
        MediaGenerator::DALLE => 5.0,
        MediaGenerator::DREAM => 10.0,
        MediaGenerator::MIST_STABILITY => 15.0,
        MediaGenerator::STABLE_HORDE => 4.0,
        _ => 10.0,
    };

    let ink_per_pixel: f64 = base_ink / (512.0 * 512.0);
    println!("ink per pixel {}", ink_per_pixel);

    let number = match number_generated {
        Some(number_generated) => number_generated,
        None => dto.number,
    };

    let pixels = ((number as u64) * (dto.width as u64 * dto.height as u64)) as u64;
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
        sqlx = sqlx.bind(ink_increase); // users.ink
        sqlx = sqlx.bind(ink_increase); // users.ink_sum
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
