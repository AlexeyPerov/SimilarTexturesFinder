use crate::config::Config;
use crate::image_loader::ImageData;
use crate::similarity::prepare::{flip_horizontal, rotate180, rotate270, rotate90};
use crate::similarity::types::{MetricResult, SimilarityMetric};

/// Max similarity over transforms of **B** only ([Task.md](../../../Tasks/Task.md) §3.1, appendix §C.6).
///
/// When `enable_rotations`: 0°, 90°, 180°, 270°. When `enable_flip`: additionally horizontal flip,
/// vertical flip, flip then 90°, flip then 180°, flip then 270° (8 transforms when both on: 4 rotations + 4 flipped rotations; flips alone add h/v flips when rotations off).
pub fn max_over_b<M: SimilarityMetric + ?Sized>(
    metric: &M,
    img_a: &ImageData,
    img_b: &ImageData,
    cfg: &Config,
) -> MetricResult {
    if !cfg.enable_rotations && !cfg.enable_flip {
        return metric.compute(img_a, img_b);
    }

    let mut best = MetricResult {
        score: -1.0,
        raw: 0.0,
        valid: false,
    };

    for bt in transforms_of_b(img_b, cfg) {
        let r = metric.compute(img_a, &bt);
        if r.valid && r.score > best.score {
            best = r;
        }
    }

    if !best.valid {
        return MetricResult {
            score: 0.0,
            raw: 0.0,
            valid: false,
        };
    }
    best
}

fn transforms_of_b(b: &ImageData, cfg: &Config) -> Vec<ImageData> {
    let rots: Vec<ImageData> = if cfg.enable_rotations {
        vec![
            b.clone(),
            rotate90(b),
            rotate180(b),
            rotate270(b),
        ]
    } else {
        vec![b.clone()]
    };

    if !cfg.enable_flip {
        return rots;
    }

    let mut out = Vec::with_capacity(rots.len() * 2);
    for r in rots {
        out.push(r.clone());
        out.push(flip_horizontal(&r));
    }
    out
}
