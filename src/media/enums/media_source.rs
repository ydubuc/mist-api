use super::media_generator::MediaGenerator;

pub enum MediaSource {
    Mist,
    StableHorde,
    Dalle,
    Import,
}

impl MediaSource {
    pub fn value(&self) -> String {
        match *self {
            MediaSource::Mist => MediaGenerator::MIST.to_string(),
            MediaSource::StableHorde => MediaGenerator::STABLE_HORDE.to_string(),
            MediaSource::Dalle => MediaGenerator::DALLE.to_string(),
            MediaSource::Import => "import".to_string(),
        }
    }
}
