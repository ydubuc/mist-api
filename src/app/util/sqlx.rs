use std::borrow::Cow;

use reqwest::StatusCode;
use sqlx::{error::DatabaseError, PgPool, Postgres, Transaction};
use tokio_retry::{strategy::FixedInterval, Retry};

use crate::app::models::api_error::ApiError;

#[non_exhaustive]
pub struct SqlStateCodes;

impl SqlStateCodes {
    pub const UNIQUE_VIOLATION: &str = "23505";
}

pub fn get_code_from_db_err(db_err: &dyn DatabaseError) -> Option<String> {
    match db_err.code() {
        Some(code) => match code {
            Cow::Borrowed(val) => Some(val.to_owned()),
            Cow::Owned(val) => Some(val),
        },
        None => None,
    }
}

// pub async fn aquire_tx_with_retry(pool: &PgPool) -> Result<Transaction<Postgres>, ApiError> {
//     let retry_strategy = FixedInterval::from_millis(10000).take(3);

//     Retry::spawn(retry_strategy, || async { aquire_tx(pool).await }).await
// }

// async fn aquire_tx(pool: &PgPool) -> Result<Transaction<Postgres>, ApiError> {
//     match pool.begin().await {
//         Ok(tx) => Ok(tx),
//         Err(e) => {
//             tracing::error!("aquire_tx: {:?}", e);
//             Err(ApiError {
//                 code: StatusCode::INTERNAL_SERVER_ERROR,
//                 message: "Failed to begin pool transaction.".to_string(),
//             })
//         }
//     }
// }
