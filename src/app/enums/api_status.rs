#[derive(Clone)]
pub enum ApiStatus {
    Online,
    Maintenance,
}

impl ApiStatus {
    pub fn value(&self) -> String {
        match *self {
            Self::Online => "online".to_string(),
            Self::Maintenance => "maintenance".to_string(),
        }
    }
}
