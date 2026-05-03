use image::RgbaImage;

use crate::config::Config;
use crate::image_loader::ImageData;
use crate::similarity::prepare::resize_square_rgba;
use crate::similarity::types::{MetricResult, SimilarityMetric};

#[derive(Clone, Copy, Debug)]
pub struct HistogramMetric {
    pub resize: u32,
    pub bins: u32,
    pub method: HistMethod,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HistMethod {
    Correlation,
    Bhattacharyya,
}

impl HistogramMetric {
    pub fn from_config(cfg: &Config) -> Self {
        let bins = histogram_bins_per_channel(cfg.hist_bins);
        let method = parse_method(&cfg.hist_method);
        Self {
            resize: cfg.resize_size,
            bins,
            method,
        }
    }
}

fn parse_method(s: &str) -> HistMethod {
    match s.to_ascii_lowercase().as_str() {
        "bhattacharyya" | "bhat" | "bhatt" => HistMethod::Bhattacharyya,
        _ => HistMethod::Correlation,
    }
}

pub fn histogram_bins_per_channel(hist_bins: u32) -> u32 {
    let b = (hist_bins as f32).cbrt().floor() as u32;
    b.max(2)
}

impl SimilarityMetric for HistogramMetric {
    fn compute(&self, img_a: &ImageData, img_b: &ImageData) -> MetricResult {
        if img_a.width == 0 || img_a.height == 0 || img_b.width == 0 || img_b.height == 0 {
            return MetricResult {
                score: 0.0,
                raw: 0.0,
                valid: false,
            };
        }

        let ra = resize_square_rgba(img_a, self.resize).to_rgba_image();
        let rb = resize_square_rgba(img_b, self.resize).to_rgba_image();
        let ha = build_rgb_hist(&ra, self.bins);
        let hb = build_rgb_hist(&rb, self.bins);

        match self.method {
            HistMethod::Bhattacharyya => {
                let bc: f32 = ha
                    .iter()
                    .zip(hb.iter())
                    .map(|(&a, &b)| a.sqrt() * b.sqrt())
                    .sum();
                let bc = bc.clamp(0.0, 1.0);
                MetricResult {
                    score: bc,
                    raw: bc,
                    valid: true,
                }
            }
            HistMethod::Correlation => {
                let score = pearson_similarity(&ha, &hb);
                if !score.is_finite() {
                    MetricResult {
                        score: 0.0,
                        raw: 0.0,
                        valid: false,
                    }
                } else {
                    let s = ((score + 1.0) * 0.5).clamp(0.0, 1.0);
                    MetricResult {
                        score: s,
                        raw: score,
                        valid: true,
                    }
                }
            }
        }
    }
}

fn build_rgb_hist(img: &RgbaImage, bins: u32) -> Vec<f32> {
    let b = bins as usize;
    let n = b * b * b;
    let mut h = vec![0f32; n];
    let scale = (b as f32) / 256.0;
    for p in img.pixels() {
        let [r, g, bl, a] = p.0;
        if a == 0 {
            continue;
        }
        let ir = ((r as f32) * scale).floor() as usize;
        let ig = ((g as f32) * scale).floor() as usize;
        let ib = ((bl as f32) * scale).floor() as usize;
        let ir = ir.min(b - 1);
        let ig = ig.min(b - 1);
        let ib = ib.min(b - 1);
        h[((ir * b + ig) * b) + ib] += 1.0;
    }
    let sum: f32 = h.iter().sum();
    if sum > 0.0 {
        for v in &mut h {
            *v /= sum;
        }
    }
    h
}

fn pearson_similarity(p: &[f32], q: &[f32]) -> f32 {
    let n = p.len() as f32;
    let mp = p.iter().copied().sum::<f32>() / n;
    let mq = q.iter().copied().sum::<f32>() / n;
    let vp = p.iter().map(|&v| (v - mp) * (v - mp)).sum::<f32>();
    let vq = q.iter().map(|&v| (v - mq) * (v - mq)).sum::<f32>();
    if vp <= 1e-12 || vq <= 1e-12 {
        return f32::NAN;
    }
    let cov = p
        .iter()
        .zip(q.iter())
        .map(|(&a, &b)| (a - mp) * (b - mq))
        .sum::<f32>();
    cov / vp.sqrt() / vq.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bins_mapping_512_to_8() {
        assert_eq!(histogram_bins_per_channel(512), 8);
    }
}
