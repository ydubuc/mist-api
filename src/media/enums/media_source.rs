pub enum MediaSource {
    Dalle,
    Import,
}

impl MediaSource {
    pub fn value(&self) -> String {
        match *self {
            MediaSource::Dalle => "dalle".to_string(),
            MediaSource::Import => "import".to_string(),
        }
    }
}
