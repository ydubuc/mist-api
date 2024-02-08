use axum::extract::Multipart;
use uuid::Uuid;

use super::models::file_properties::FileProperties;

pub async fn get_files_properties(mut multipart: Multipart) -> Vec<FileProperties> {
    let mut vec = Vec::new();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let field_name = field.name().unwrap_or("file").to_string();
        let file_name = field.file_name().unwrap_or("file-name").to_string();
        let mime_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();
        let Ok(data) = field.bytes().await else {
            continue;
        };

        let properties = FileProperties {
            id: Uuid::new_v4().to_string(),
            field_name,
            file_name,
            mime_type,
            data,
        };

        vec.push(properties);
    }

    vec
}
