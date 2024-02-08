#[non_exhaustive]
pub struct PepperType;

impl PepperType {
    pub const EDIT_PASSWORD: &'static str = "edit-password";
    pub const EDIT_EMAIL: &'static str = "edit-email";
    pub const VERIFY_EMAIL: &'static str = "verify-email";
}
