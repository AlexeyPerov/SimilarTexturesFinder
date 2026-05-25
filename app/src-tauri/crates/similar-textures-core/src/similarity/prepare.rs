use image::imageops::{resize, FilterType};
use image::DynamicImage;

use crate::config::Config;
use crate::features::alpha_crop;
use crate::image_loader::ImageData;

pub fn prepare_vertex(img: &ImageData, cfg: &Config) -> Result<ImageData, String> {
    if cfg.enable_alpha_crop {
        alpha_crop::apply(img, cfg.alpha_threshold)
    } else {
        Ok(img.clone())
    }
}

pub fn resize_square_rgba(img: &ImageData, size: u32) -> ImageData {
    let src = img.to_rgba_image();
    let scaled = resize(&src, size, size, FilterType::Triangle);
    ImageData::from_rgba_image(scaled)
}

/// ITU-R BT.601 luma weighted by normalized alpha (transparent pixels contribute black).
pub fn luma_at_pixel(r: f32, g: f32, b: f32, a: f32) -> f32 {
    let l = 0.299 * r + 0.587 * g + 0.114 * b;
    l * (a / 255.0)
}

/// Luma (ITU-R BT.601) per pixel, row-major.
pub fn luma_plane(img: &ImageData) -> Vec<f32> {
    let w = img.width as usize;
    let h = img.height as usize;
    let rgba = img.rgba();
    let mut out = Vec::with_capacity(w * h);
    let stride = w * 4;
    for y in 0..h {
        for x in 0..w {
            let i = y * stride + x * 4;
            let r = rgba[i] as f32;
            let g = rgba[i + 1] as f32;
            let b = rgba[i + 2] as f32;
            let a = rgba[i + 3] as f32;
            out.push(luma_at_pixel(r, g, b, a));
        }
    }
    out
}

pub fn rotate90(d: &ImageData) -> ImageData {
    let di = DynamicImage::ImageRgba8(d.to_rgba_image());
    ImageData::from_rgba_image(di.rotate90().to_rgba8())
}

pub fn rotate180(d: &ImageData) -> ImageData {
    let di = DynamicImage::ImageRgba8(d.to_rgba_image());
    ImageData::from_rgba_image(di.rotate180().to_rgba8())
}

pub fn rotate270(d: &ImageData) -> ImageData {
    let di = DynamicImage::ImageRgba8(d.to_rgba_image());
    ImageData::from_rgba_image(di.rotate270().to_rgba8())
}

pub fn flip_horizontal(d: &ImageData) -> ImageData {
    ImageData::from_rgba_image(image::imageops::flip_horizontal(&d.to_rgba_image()))
}
