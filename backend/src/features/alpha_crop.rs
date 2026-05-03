use std::cmp::{max, min};

use image::imageops::crop_imm;

use crate::image_loader::ImageData;

/// Crop to the bounding box of pixels with normalized alpha `> threshold` ([Task_similarity_appendix.md](../../../Tasks/Task_similarity_appendix.md) §C.4).
/// Fully transparent images return `Err`.
pub fn apply(img: &ImageData, alpha_threshold: f64) -> Result<ImageData, String> {
    let w = img.width;
    let h = img.height;
    if w == 0 || h == 0 {
        return Err("empty image".to_string());
    }

    let rgba = img.rgba();
    let thr = alpha_threshold as f32;
    let mut min_x = w;
    let mut min_y = h;
    let mut max_x = 0u32;
    let mut max_y = 0u32;
    let mut any = false;

    for y in 0..h {
        for x in 0..w {
            let a = rgba[((y * w + x) as usize) * 4 + 3] as f32 / 255.0;
            if a > thr {
                any = true;
                min_x = min(min_x, x);
                min_y = min(min_y, y);
                max_x = max(max_x, x);
                max_y = max(max_y, y);
            }
        }
    }

    if !any {
        return Err("fully transparent (alpha crop)".to_string());
    }

    let full = img.to_rgba_image();
    let cw = max_x - min_x + 1;
    let ch = max_y - min_y + 1;
    let sub = crop_imm(&full, min_x, min_y, cw, ch);
    Ok(ImageData::from_rgba_image(sub.to_image()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    #[test]
    fn opaque_full_frame() {
        let mut img = RgbaImage::new(10, 10);
        for p in img.pixels_mut() {
            *p = Rgba([10, 20, 30, 255]);
        }
        let d = ImageData::from_rgba_image(img);
        let out = apply(&d, 0.05).unwrap();
        assert_eq!((out.width, out.height), (10, 10));
    }

    #[test]
    fn crops_nontransparent_bbox() {
        let mut img = RgbaImage::new(20, 20);
        for p in img.pixels_mut() {
            *p = Rgba([0, 0, 0, 0]);
        }
        for y in 5..10 {
            for x in 6..12 {
                img.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }
        let d = ImageData::from_rgba_image(img);
        let out = apply(&d, 0.05).unwrap();
        assert_eq!((out.width, out.height), (6, 5));
    }
}
