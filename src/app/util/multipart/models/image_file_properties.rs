use imagesize::ImageSize;

use super::file_properties::FileProperties;

#[derive(Debug)]
pub struct ImageFileProperties {
    pub file_properties: FileProperties,
    pub image_size: ImageSize,
}
