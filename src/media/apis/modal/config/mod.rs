// pub static API_URL: &str = "https://ydubuc--mist-modal-entrypoint.modal.run";
// pub static API_URL: &str = "https://ydubuc--mist-dreamlike-art-dreamlike-diffusion-1-0-entrypoint.modal.run"

use crate::media::enums::media_model::MediaModel;

pub fn api_url(model: &str) -> &str {
    match model {
        // MediaModel::STABLE_DIFFUSION_1_5 => {
        //     "https://ydubuc--mist-dreamlike-art-dreamlike-diffusion-1-0-entrypoint.modal.run"
        // }
        // MediaModel::STABLE_DIFFUSION_2_1 => {
        //     "https://ydubuc--mist-dreamlike-art-dreamlike-diffusion-1-0-entrypoint.modal.run"
        // }
        MediaModel::OPENJOURNEY => {
            "https://ydubuc--mist-prompthero-openjourney-entrypoint.modal.run"
        }
        _ => "",
    }
}
