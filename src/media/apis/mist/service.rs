use crate::media::enums::media_model::MediaModel;

pub fn is_valid_model(model: &str) -> bool {
    let valid_models: [&str; 3] = [
        MediaModel::OPENJOURNEY,
        MediaModel::STABLE_DIFFUSION_1_5,
        MediaModel::STABLE_DIFFUSION_2_1,
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

pub fn is_valid_number(number: u8, model: &str) -> bool {
    match model {
        MediaModel::OPENJOURNEY => number == 1 || number == 4,
        MediaModel::STABLE_DIFFUSION_1_5 => number > 0 && number < 5,
        MediaModel::STABLE_DIFFUSION_2_1 => number > 0 && number < 5,
        _ => false,
    }
}
