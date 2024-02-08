#[non_exhaustive]
pub struct DreamTaskState;

impl DreamTaskState {
    pub const INPUT: &'static str = "input";
    pub const PENDING: &'static str = "pending";
    pub const GENERATING: &'static str = "generating";
    pub const COMPLETED: &'static str = "completed";
}
