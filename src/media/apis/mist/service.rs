use crate::media::enums::media_model::MediaModel;

pub fn is_valid_model(model: &str) -> bool {
    let valid_models: [&str; 7] = [
        MediaModel::STABLE_DIFFUSION_1_5,
        MediaModel::STABLE_DIFFUSION_2_1,
        MediaModel::OPENJOURNEY,
        MediaModel::OPENJOURNEY_2,
        MediaModel::DREAMSHAPER,
        MediaModel::DREAMLIKE_DIFFUSION_1,
        MediaModel::ARCANE_DIFFUSION,
    ];

    return valid_models.contains(&model);
}

pub fn is_valid_size(width: &u16, height: &u16, _model: &str) -> bool {
    let valid_widths: [u16; 3] = [512, 768, 1024];

    if !valid_widths.contains(width) {
        return false;
    }

    let valid_heights: [u16; 3] = [512, 768, 1024];

    if !valid_heights.contains(height) {
        return false;
    }

    if *width == 1024 && *height == 1024 {
        return false;
    }

    return true;
}

pub fn is_valid_number(number: u8, _model: &str) -> bool {
    return number > 0 && number < 7;
}
