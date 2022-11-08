use imagesize::ImageSize;

use crate::media::util::backblaze::models::backblaze_upload_file_response::BackblazeUploadFileResponse;

#[derive(Debug)]
pub struct ImportMediaResponse {
    pub id: String,
    pub download_url: String,
    pub size: ImageSize,
    pub backblaze_upload_file_response: BackblazeUploadFileResponse,
}
