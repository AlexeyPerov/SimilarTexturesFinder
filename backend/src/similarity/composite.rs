use crate::config::Config;
use crate::similarity::types::MetricResult;

/// §5 weighted `final_score` over enabled metrics that returned `valid` for this pair.
pub fn combine(
    cfg: &Config,
    ph: Option<MetricResult>,
    ss: Option<MetricResult>,
    hi: Option<MetricResult>,
) -> Option<f64> {
    let mut w_sum = 0.0_f64;
    let mut acc = 0.0_f64;

    if cfg.enable_phash {
        if let Some(r) = ph {
            if r.valid {
                let w = cfg.weights.phash;
                if w > 0.0 {
                    acc += w * f64::from(r.score);
                    w_sum += w;
                }
            }
        }
    }

    if cfg.enable_ssim {
        if let Some(r) = ss {
            if r.valid {
                let w = cfg.weights.ssim;
                if w > 0.0 {
                    acc += w * f64::from(r.score);
                    w_sum += w;
                }
            }
        }
    }

    if cfg.enable_histogram {
        if let Some(r) = hi {
            if r.valid {
                let w = cfg.weights.histogram;
                if w > 0.0 {
                    acc += w * f64::from(r.score);
                    w_sum += w;
                }
            }
        }
    }

    if w_sum == 0.0 {
        None
    } else {
        Some(acc / w_sum)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn cfg_all_on() -> Config {
        let json = r#"{
            "enable_phash": true,
            "enable_ssim": true,
            "enable_histogram": true,
            "threshold": 0.5,
            "weights": { "phash": 0.35, "ssim": 0.45, "histogram": 0.2 }
        }"#;
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn renormalizes_when_one_invalid() {
        let cfg = cfg_all_on();
        let ph = MetricResult {
            score: 1.0,
            raw: 0.0,
            valid: true,
        };
        let ss = MetricResult {
            score: 0.0,
            raw: 0.0,
            valid: false,
        };
        let hi = MetricResult {
            score: 0.0,
            raw: 0.0,
            valid: true,
        };
        let fs = combine(&cfg, Some(ph), Some(ss), Some(hi)).unwrap();
        let wp = cfg.weights.phash;
        let wh = cfg.weights.histogram;
        let exp = (wp * 1.0 + wh * 0.0) / (wp + wh);
        assert!((fs - exp).abs() < 1e-9);
    }

    #[test]
    fn none_when_all_invalid() {
        let cfg = cfg_all_on();
        let r = MetricResult {
            score: 0.9,
            raw: 0.0,
            valid: false,
        };
        assert!(combine(&cfg, Some(r), Some(r), Some(r)).is_none());
    }
}
