use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ModalEntrypointResponse {
    pub request_id: String,
}
