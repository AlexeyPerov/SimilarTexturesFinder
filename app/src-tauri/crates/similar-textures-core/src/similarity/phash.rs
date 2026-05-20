use rustdct::DctPlanner;

use crate::image_loader::ImageData;
use crate::similarity::types::{MetricResult, SimilarityMetric};

const PHASH_SIZE: u32 = 32;
const PHASH_LOW: usize = 8;

/// pHash per appendix §C.2: grayscale → 32×32 → DCT → 8×8 low block → median → 64-bit hash.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub struct PHashMetric {
    pub max_hamming_distance: u32,
}

impl PHashMetric {
    #[allow(dead_code)]
    pub fn from_config(cfg: &crate::config::Config) -> Self {
        Self {
            max_hamming_distance: cfg.phash_max_distance,
        }
    }
}

pub(crate) fn compute_hash(img: &ImageData) -> u64 {
    let luma = resize_luma_32(img);
    let dct = dct_2d_32(&luma);
    let mut block = [0f32; 64];
    let mut k = 0;
    for y in 0..PHASH_LOW {
        for x in 0..PHASH_LOW {
            block[k] = dct[y * 32 + x];
            k += 1;
        }
    }
    let median = median_64(&block);
    let mut bits: u64 = 0;
    for (i, &v) in block.iter().enumerate() {
        if v > median {
            bits |= 1u64 << i;
        }
    }
    bits
}

pub(crate) fn score_hashes(a: u64, b: u64, max_dist: u32) -> MetricResult {
    let dist = (a ^ b).count_ones() as f32;
    let mut score = 1.0 - (dist / 64.0);
    if dist > max_dist as f32 {
        score = 0.0;
    }
    MetricResult {
        score,
        raw: dist,
        valid: true,
    }
}

impl SimilarityMetric for PHashMetric {
    fn compute(&self, img_a: &ImageData, img_b: &ImageData) -> MetricResult {
        if img_a.width == 0 || img_a.height == 0 {
            return MetricResult {
                score: 0.0,
                raw: 0.0,
                valid: false,
            };
        }
        if img_b.width == 0 || img_b.height == 0 {
            return MetricResult {
                score: 0.0,
                raw: 0.0,
                valid: false,
            };
        }

        let ha = compute_hash(img_a);
        let hb = compute_hash(img_b);
        score_hashes(ha, hb, self.max_hamming_distance)
    }
}

fn resize_luma_32(img: &ImageData) -> Vec<f32> {
    let sw = img.width as f32;
    let sh = img.height as f32;
    let mut out = vec![0f32; (PHASH_SIZE * PHASH_SIZE) as usize];
    let rgba = img.rgba();
    for ty in 0..PHASH_SIZE {
        for tx in 0..PHASH_SIZE {
            let sx = ((tx as f32 + 0.5) * sw / PHASH_SIZE as f32).min(sw - 1.0);
            let sy = ((ty as f32 + 0.5) * sh / PHASH_SIZE as f32).min(sh - 1.0);
            let x0 = sx.floor() as u32;
            let y0 = sy.floor() as u32;
            let x1 = (x0 + 1).min(img.width.saturating_sub(1));
            let y1 = (y0 + 1).min(img.height.saturating_sub(1));
            let fx = sx - x0 as f32;
            let fy = sy - y0 as f32;

            let l00 = luma_at(rgba, img.width, x0, y0);
            let l10 = luma_at(rgba, img.width, x1, y0);
            let l01 = luma_at(rgba, img.width, x0, y1);
            let l11 = luma_at(rgba, img.width, x1, y1);

            let l0 = l00 * (1.0 - fx) + l10 * fx;
            let l1 = l01 * (1.0 - fx) + l11 * fx;
            let l = l0 * (1.0 - fy) + l1 * fy;
            out[(ty * PHASH_SIZE + tx) as usize] = l;
        }
    }
    out
}

fn luma_at(rgba: &[u8], width: u32, x: u32, y: u32) -> f32 {
    let i = ((y * width + x) as usize) * 4;
    let r = rgba[i] as f32;
    let g = rgba[i + 1] as f32;
    let b = rgba[i + 2] as f32;
    0.299 * r + 0.587 * g + 0.114 * b
}

fn dct_2d_32(input: &[f32]) -> Vec<f32> {
    let mut buf = input.to_vec();
    let mut planner = DctPlanner::<f32>::new();
    let row_dct = planner.plan_dct2(32);

    for r in 0..32 {
        let start = r * 32;
        let row = &mut buf[start..start + 32];
        row_dct.process_dct2(row);
    }

    let col_dct = planner.plan_dct2(32);
    let mut col = vec![0f32; 32];
    for c in 0..32 {
        for r in 0..32 {
            col[r] = buf[r * 32 + c];
        }
        col_dct.process_dct2(&mut col);
        for r in 0..32 {
            buf[r * 32 + c] = col[r];
        }
    }

    buf
}

fn median_64(block: &[f32; 64]) -> f32 {
    let mut s = *block;
    s.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    (s[31] + s[32]) * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    #[test]
    fn identical_image_score_one() {
        let mut img = RgbaImage::new(64, 64);
        for p in img.pixels_mut() {
            *p = Rgba([100, 150, 80, 255]);
        }
        let data = ImageData::from_rgba_image(img);
        let m = PHashMetric {
            max_hamming_distance: 10,
        };
        let r = m.compute(&data, &data);
        assert!(r.valid);
        assert!((r.score - 1.0).abs() < 1e-4, "score={}", r.score);
    }
}
