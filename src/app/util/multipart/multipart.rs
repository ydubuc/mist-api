use axum::extract::Multipart;
use mime::Mime;

use super::models::file_properties::FileProperties;

pub async fn get_files_properties(mut multipart: Multipart) -> Vec<FileProperties> {
    let mut vec = Vec::new();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let field_name = field.name().unwrap_or("file").to_string();
        let file_name = field.file_name().unwrap_or("file-name").to_string();
        let mime_type: Mime = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string()
            .parse()
            .unwrap();
        let Ok(data) = field.bytes().await else { continue };

        let properties = FileProperties {
            field_name,
            file_name,
            mime_type,
            data,
        };

        vec.push(properties);
    }

    vec

    // match multipart.next_field().await {
    //     Ok(field) => {
    //         let Some(field) = field else { return None };

    //         let field_name = field.name()?.to_string();
    //         let file_name = field.file_name()?.to_string();
    //         let mime_type: Mime = field.content_type()?.to_string().parse().unwrap();

    //         let Ok(data) = field.bytes().await else { return None };

    //         Some(FileProperties {
    //             field_name,
    //             file_name,
    //             mime_type,
    //             data,
    //         })
    //     }
    //     Err(e) => {
    //         tracing::error!(%e);
    //         None
    //     }
    // }
}
