#[derive(Debug)]
pub enum GenerateMediaRequestStatus {
    Pending,
    Processing,
    Completed,
    Canceled,
    Error,
}

impl GenerateMediaRequestStatus {
    pub fn value(&self) -> &str {
        match *self {
            Self::Pending => "pending",
            Self::Processing => "processing",
            Self::Completed => "completed",
            Self::Canceled => "canceled",
            Self::Error => "error",
        }
    }
}
