use bytes::Bytes;
use mime::Mime;

#[derive(Debug)]
pub struct FileProperties {
    pub field_name: String,
    pub file_name: String,
    pub mime_type: Mime,
    pub data: Bytes,
}
