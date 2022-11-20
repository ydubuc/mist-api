use bytes::Bytes;

#[derive(Debug)]
pub struct FileProperties {
    pub id: String,
    pub field_name: String,
    pub file_name: String,
    pub mime_type: String,
    pub data: Bytes,
}
