use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub user_id: String,
    pub data: sqlx::types::JsonValue,
    pub created_at: i64,
}
