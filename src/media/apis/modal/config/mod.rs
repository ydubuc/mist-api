// pub static API_URL: &str = "https://ydubuc--mist-modal-entrypoint.modal.run";
// pub static API_URL: &str = "https://ydubuc--mist-dreamlike-art-dreamlike-diffusion-1-0-entrypoint.modal.run"

use crate::media::enums::media_model::MediaModel;

pub fn api_url(model: &str) -> &str {
    match model {
        MediaModel::STABLE_DIFFUSION_1_5 => {
            "https://ydubuc--mist-runwayml-stable-diffusion-v1-5-entrypoint.modal.run"
        }
        MediaModel::STABLE_DIFFUSION_2_1 => {
            "https://ydubuc--mist-stabilityai-stable-diffusion-2-1-entrypoint.modal.run"
        }
        MediaModel::OPENJOURNEY => {
            "https://ydubuc--mist-prompthero-openjourney-entrypoint.modal.run"
        }
        MediaModel::OPENJOURNEY_4 => {
            "https://ydubuc--mist-prompthero-openjourney-v4-entrypoint.modal.run"
        }
        MediaModel::DREAMSHAPER => "https://ydubuc--mist-lykon-dreamshaper-entrypoint.modal.run",
        MediaModel::DREAMLIKE_DIFFUSION_1 => {
            "https://ydubuc--mist-dreamlike-art-dreamlike-diffusion-1-0-entrypoint.modal.run"
        }
        MediaModel::ARCANE_DIFFUSION => {
            "https://ydubuc--mist-nitrosocke-arcane-diffusion-entrypoint.modal.run"
        }
        _ => "",
    }
}
