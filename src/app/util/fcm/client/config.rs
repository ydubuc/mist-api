#[derive(Debug, Clone)]
pub struct Config {
    pub project_name: String,
    pub client_email: String,
    pub private_key: String,
}

impl Config {
    pub fn new(project_name: String, client_email: String, private_key: String) -> Config {
        Config {
            project_name,
            client_email,
            private_key,
        }
    }
}
