use crate::config::Config;
use crate::image_loader::ImageData;
use crate::similarity::histogram;
use crate::similarity::phash;
use crate::similarity::prepare::{flip_horizontal, luma_plane, resize_square_rgba, rotate90, rotate180, rotate270};

use super::{Features, ImageFeatures};

pub fn extract_features(img: &ImageData, cfg: &Config) -> Result<Features, String> {
    let base = extract_single(img, cfg)?;

    if !cfg.enable_rotations && !cfg.enable_flip {
        return Ok(Features {
            transforms: vec![base],
        });
    }

    let mut all = Vec::new();
    all.push(base);

    let rots: Vec<ImageData> = if cfg.enable_rotations {
        vec![rotate90(img), rotate180(img), rotate270(img)]
    } else {
        vec![]
    };

    for r in &rots {
        all.push(extract_single(r, cfg)?);
    }

    if cfg.enable_flip {
        let flipped_identity = flip_horizontal(img);
        all.push(extract_single(&flipped_identity, cfg)?);
        for r in &rots {
            let flipped = flip_horizontal(r);
            all.push(extract_single(&flipped, cfg)?);
        }
    }

    Ok(Features { transforms: all })
}

fn extract_single(img: &ImageData, cfg: &Config) -> Result<ImageFeatures, String> {
    if img.width == 0 || img.height == 0 {
        return Err("empty image".to_string());
    }
    let ph = phash::compute_hash(img);
    let resized = resize_square_rgba(img, cfg.resize_size);
    let luma = luma_plane(&resized);
    let bins = histogram::histogram_bins_per_channel(cfg.hist_bins);
    let hist = histogram::build_hist(&resized.to_rgba_image(), bins);
    Ok(ImageFeatures {
        phash: ph,
        ssim_luma: luma,
        histogram: hist,
    })
}
