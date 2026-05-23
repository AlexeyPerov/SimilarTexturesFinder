use std::path::Path;

use image::{imageops::FilterType, DynamicImage, GenericImageView, RgbaImage};

/// Raster input for similarity metrics: RGBA8, row-major `rgba` bytes.
#[derive(Clone, Debug)]
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    rgba: Vec<u8>,
}

impl ImageData {
    #[allow(dead_code)]
    pub fn new(width: u32, height: u32, rgba: Vec<u8>) -> Option<Self> {
        let expected = (width as usize)
            .checked_mul(height as usize)?
            .checked_mul(4)?;
        if rgba.len() != expected {
            return None;
        }
        Some(Self {
            width,
            height,
            rgba,
        })
    }

    pub fn rgba(&self) -> &[u8] {
        &self.rgba
    }

    pub fn from_dynamic(image: DynamicImage, max_dimension_px: Option<u32>) -> Self {
        let mut img = image;
        if let Some(max_dim) = max_dimension_px {
            let (w, h) = img.dimensions();
            let longest = w.max(h);
            if longest > max_dim {
                let scale = max_dim as f64 / longest as f64;
                let nw = ((w as f64) * scale).round().max(1.0) as u32;
                let nh = ((h as f64) * scale).round().max(1.0) as u32;
                img = DynamicImage::ImageRgba8(img.to_rgba8()).resize(nw, nh, FilterType::Triangle);
            }
        }

        let rgba = img.to_rgba8();
        Self::from_rgba_image(rgba)
    }

    pub fn from_rgba_image(rgba: RgbaImage) -> Self {
        let (width, height) = rgba.dimensions();
        Self {
            width,
            height,
            rgba: rgba.into_raw(),
        }
    }

    /// Decode image at `path`; on failure returns `Err(reason)`.
    pub fn decode_path(path: &Path, max_dimension_px: Option<u32>) -> Result<Self, String> {
        let img = image::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(Self::from_dynamic(img, max_dimension_px))
    }

    /// Write a JPEG preview capped to `max_dimension_px` (longest side).
    pub fn write_preview_jpeg(
        source: &Path,
        dest: &Path,
        max_dimension_px: u32,
    ) -> Result<(), String> {
        let data = Self::decode_path(source, Some(max_dimension_px))?;
        let rgba = data.to_rgba_image();
        let mut bytes: Vec<u8> = Vec::new();
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, 85);
        encoder
            .encode(
                rgba.as_raw(),
                rgba.width(),
                rgba.height(),
                image::ExtendedColorType::Rgba8,
            )
            .map_err(|e| format!("encode preview for {}: {e}", source.display()))?;
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("create preview cache dir {}: {e}", parent.display()))?;
        }
        std::fs::write(dest, bytes)
            .map_err(|e| format!("write preview {}: {e}", dest.display()))
    }

    pub fn to_rgba_image(&self) -> RgbaImage {
        RgbaImage::from_raw(self.width, self.height, self.rgba.clone())
            .expect("rgba length matches dimensions")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    #[test]
    fn downscales_to_max_dimension() {
        let mut img = RgbaImage::new(400, 200);
        for p in img.pixels_mut() {
            *p = Rgba([10, 20, 30, 255]);
        }
        let data = ImageData::from_dynamic(DynamicImage::ImageRgba8(img), Some(100));
        assert_eq!(data.width.max(data.height), 100);
    }
}
