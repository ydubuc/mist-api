pub enum MediaSource {
    Dalle,
    Dream,
    StableHorde,
    MistStability,
    Import,
}

impl MediaSource {
    pub fn value(&self) -> String {
        match *self {
            MediaSource::Dalle => "dalle".to_string(),
            MediaSource::Dream => "dream".to_string(),
            MediaSource::StableHorde => "stable_horde".to_string(),
            MediaSource::MistStability => "mist_stability".to_string(),
            MediaSource::Import => "import".to_string(),
        }
    }
}
