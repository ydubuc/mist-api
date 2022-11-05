#[non_exhaustive]
pub struct Env;

impl Env {
    pub const PORT: &str = "PORT";
    pub const DATABASE_URL: &str = "DATABASE_URL";
    pub const JWT_SECRET: &str = "JWT_SECRET";
    pub const OPENAI_API_KEY: &str = "OPENAI_API_KEY";
}
