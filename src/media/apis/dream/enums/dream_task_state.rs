#[non_exhaustive]
pub struct DreamTaskState;

impl DreamTaskState {
    pub const INPUT: &str = "input";
    pub const PENDING: &str = "pending";
    pub const GENERATING: &str = "generating";
    pub const COMPLETED: &str = "completed";
}
