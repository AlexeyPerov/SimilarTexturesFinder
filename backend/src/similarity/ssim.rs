use crate::image_loader::ImageData;
use crate::similarity::prepare::{luma_plane, resize_square_rgba};
use crate::similarity::types::{MetricResult, SimilarityMetric};

/// Global (single-window) SSIM on 8-bit luma scale ([Task_similarity_appendix.md](../../../Tasks/Task_similarity_appendix.md) §C.3).
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub struct SsimMetric {
    pub resize: u32,
    pub min_ssim: f64,
}

impl SsimMetric {
    #[allow(dead_code)]
    pub fn from_config(cfg: &crate::config::Config) -> Self {
        Self {
            resize: cfg.resize_size,
            min_ssim: cfg.ssim_threshold,
        }
    }
}

impl SimilarityMetric for SsimMetric {
    fn compute(&self, img_a: &ImageData, img_b: &ImageData) -> MetricResult {
        if img_a.width == 0 || img_a.height == 0 || img_b.width == 0 || img_b.height == 0 {
            return MetricResult {
                score: 0.0,
                raw: 0.0,
                valid: false,
            };
        }

        let ra = resize_square_rgba(img_a, self.resize);
        let rb = resize_square_rgba(img_b, self.resize);
        let xa = luma_plane(&ra);
        let xb = luma_plane(&rb);

        score_luma(&xa, &xb, self.min_ssim)
    }
}

pub(crate) fn score_luma(luma_a: &[f32], luma_b: &[f32], min_ssim: f64) -> MetricResult {
    let s = global_ssim(luma_a, luma_b);
    if !s.is_finite() {
        return MetricResult {
            score: 0.0,
            raw: 0.0,
            valid: false,
        };
    }

    let score = s.clamp(0.0, 1.0);
    let valid = score >= min_ssim as f32;
    MetricResult {
        score,
        raw: score,
        valid,
    }
}

fn global_ssim(x: &[f32], y: &[f32]) -> f32 {
    let n = x.len() as f32;
    if n == 0.0 {
        return f32::NAN;
    }

    let mx = x.iter().copied().sum::<f32>() / n;
    let my = y.iter().copied().sum::<f32>() / n;
    let vx = x.iter().map(|&v| (v - mx) * (v - mx)).sum::<f32>() / n;
    let vy = y.iter().map(|&v| (v - my) * (v - my)).sum::<f32>() / n;
    let vxy = x
        .iter()
        .zip(y.iter())
        .map(|(&a, &b)| (a - mx) * (b - my))
        .sum::<f32>()
        / n;

    let l = 255.0_f32;
    let c1 = (0.01_f32 * l).powi(2);
    let c2 = (0.03_f32 * l).powi(2);

    let num = (2.0 * mx * my + c1) * (2.0 * vxy + c2);
    let den = (mx * mx + my * my + c1) * (vx + vy + c2);
    if den == 0.0 {
        return f32::NAN;
    }
    num / den
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    #[test]
    fn identical_is_one_and_valid_when_threshold_low() {
        let mut img = RgbaImage::new(32, 32);
        for p in img.pixels_mut() {
            *p = Rgba([40, 80, 120, 255]);
        }
        let d = ImageData::from_rgba_image(img);
        let m = SsimMetric {
            resize: 64,
            min_ssim: 0.0,
        };
        let r = m.compute(&d, &d);
        assert!(r.valid);
        assert!((r.score - 1.0).abs() < 0.05, "score={}", r.score);
    }
}
