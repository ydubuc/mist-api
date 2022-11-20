use regex::Regex;

pub mod edit_user_dto;
pub mod get_users_filter_dto;

lazy_static! {
    pub static ref DISPLAYNAME_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_.-]{3,24}$").unwrap();
}
