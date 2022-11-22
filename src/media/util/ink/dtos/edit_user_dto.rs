use reqwest::StatusCode;

use crate::app::models::api_error::ApiError;

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
            clauses.push(["ink_sum = ink_sum + $", &index.to_string()].concat());
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
        if self.ink_pending_decrease.is_some() {
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

        tracing::debug!(sql);

        Ok(sql)
    }
}
