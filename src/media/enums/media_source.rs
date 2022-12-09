use super::media_generator::MediaGenerator;

pub enum MediaSource {
    Dalle,
    Dream,
    StableHorde,
    MistStability,
    LabML,
    Import,
}

impl MediaSource {
    pub fn value(&self) -> String {
        match *self {
            MediaSource::Dalle => MediaGenerator::DALLE.to_string(),
            MediaSource::Dream => MediaGenerator::DREAM.to_string(),
            MediaSource::StableHorde => MediaGenerator::STABLE_HORDE.to_string(),
            MediaSource::MistStability => MediaGenerator::MIST_STABILITY.to_string(),
            MediaSource::LabML => MediaGenerator::LABML.to_string(),
            MediaSource::Import => "import".to_string(),
        }
    }
}
