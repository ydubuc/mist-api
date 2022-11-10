use serde::Deserialize;
use validator::Validate;

// TODO: regex for password
// static RE_TWO_CHARS: Regex = Regex::new(r"[a-z]{2}$").unwrap();

#[derive(Debug, Deserialize, Validate)]
pub struct EditPasswordDto {
    pub password: String,
}
