pub enum ReplicatePredictionStatus {
    Starting,
    Processing,
    Succeeded,
    Failed,
    Canceled,
}

impl ReplicatePredictionStatus {
    pub fn value(&self) -> String {
        match *self {
            ReplicatePredictionStatus::Starting => "starting".to_string(),
            ReplicatePredictionStatus::Processing => "processing".to_string(),
            ReplicatePredictionStatus::Succeeded => "succeeded".to_string(),
            ReplicatePredictionStatus::Failed => "failed".to_string(),
            ReplicatePredictionStatus::Canceled => "canceled".to_string(),
        }
    }
}
