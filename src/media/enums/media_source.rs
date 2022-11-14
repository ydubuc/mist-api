pub enum MediaSource {
    Dalle,
    Dream,
    StableHorde,
    Import,
}

impl MediaSource {
    pub fn value(&self) -> String {
        match *self {
            MediaSource::Dalle => "dalle".to_string(),
            MediaSource::Dream => "dream".to_string(),
            MediaSource::StableHorde => "stable_horde".to_string(),
            MediaSource::Import => "import".to_string(),
        }
    }
}
