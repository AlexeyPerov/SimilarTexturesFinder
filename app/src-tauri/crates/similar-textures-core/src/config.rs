use serde::{Deserialize, Serialize};

use crate::file_hash::HashAlgorithm;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub enable_phash: bool,
    pub enable_ssim: bool,
    pub enable_histogram: bool,
    #[serde(default)]
    #[allow(dead_code)]
    pub enable_orb: bool,
    #[serde(default)]
    pub enable_alpha_crop: bool,
    #[serde(default)]
    pub enable_rotations: bool,
    #[serde(default)]
    pub enable_flip: bool,
    pub threshold: f64,
    pub weights: Weights,
    #[serde(default = "default_hash_algorithm")]
    pub hash_algorithm: String,
    #[serde(default = "default_phash_max_distance")]
    pub phash_max_distance: u32,
    #[serde(default = "default_ssim_threshold")]
    pub ssim_threshold: f64,
    #[serde(default = "default_resize_size")]
    pub resize_size: u32,
    #[serde(default = "default_hist_bins")]
    pub hist_bins: u32,
    #[serde(default = "default_hist_method")]
    pub hist_method: String,
    #[serde(default = "default_alpha_threshold")]
    pub alpha_threshold: f64,
    #[serde(default)]
    #[allow(dead_code)]
    pub orb_max_features: Option<u32>,
    #[serde(default)]
    #[allow(dead_code)]
    pub orb_match_threshold: Option<f64>,
    #[serde(default)]
    pub max_decode_dimension_px: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Weights {
    pub phash: f64,
    pub ssim: f64,
    pub histogram: f64,
    #[serde(default)]
    pub orb: f64,
}

fn default_hash_algorithm() -> String {
    "sha256".to_string()
}

fn default_phash_max_distance() -> u32 {
    10
}

fn default_ssim_threshold() -> f64 {
    0.9
}

fn default_resize_size() -> u32 {
    256
}

fn default_hist_bins() -> u32 {
    512
}

fn default_hist_method() -> String {
    "correlation".to_string()
}

fn default_alpha_threshold() -> f64 {
    0.05
}

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Json(serde_json::Error),
    ThresholdOutOfRange,
    NegativeWeight { key: &'static str },
    NoMetricEnabled,
    UnknownHashAlgorithm(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "{e}"),
            ConfigError::Json(e) => write!(f, "{e}"),
            ConfigError::ThresholdOutOfRange => {
                write!(f, "threshold must be between 0.0 and 1.0 inclusive")
            }
            ConfigError::NegativeWeight { key } => {
                write!(f, "weight {key} must be non-negative")
            }
            ConfigError::NoMetricEnabled => write!(
                f,
                "at least one of enable_phash, enable_ssim, enable_histogram must be true"
            ),
            ConfigError::UnknownHashAlgorithm(s) => {
                write!(f, "unknown hash_algorithm {s:?} (supported: md5, sha1, sha256)")
            }
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::Io(e) => Some(e),
            ConfigError::Json(e) => Some(e),
            _ => None,
        }
    }
}

pub fn load_config(path: &std::path::Path) -> Result<Config, ConfigError> {
    let text = std::fs::read_to_string(path).map_err(ConfigError::Io)?;
    let cfg: Config = serde_json::from_str(&text).map_err(ConfigError::Json)?;
    validate(&cfg)?;
    Ok(cfg)
}

pub fn validate(cfg: &Config) -> Result<(), ConfigError> {
    if !(0.0..=1.0).contains(&cfg.threshold) {
        return Err(ConfigError::ThresholdOutOfRange);
    }

    if cfg.weights.phash < 0.0 {
        return Err(ConfigError::NegativeWeight { key: "phash" });
    }
    if cfg.weights.ssim < 0.0 {
        return Err(ConfigError::NegativeWeight { key: "ssim" });
    }
    if cfg.weights.histogram < 0.0 {
        return Err(ConfigError::NegativeWeight { key: "histogram" });
    }
    if cfg.weights.orb < 0.0 {
        return Err(ConfigError::NegativeWeight { key: "orb" });
    }

    if !cfg.enable_phash && !cfg.enable_ssim && !cfg.enable_histogram {
        return Err(ConfigError::NoMetricEnabled);
    }

    HashAlgorithm::parse(&cfg.hash_algorithm).map_err(ConfigError::UnknownHashAlgorithm)?;

    Ok(())
}

impl Config {
    pub fn hash_algorithm(&self) -> Result<HashAlgorithm, String> {
        HashAlgorithm::parse(&self.hash_algorithm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_example_loads() {
        let json = r#"{
            "enable_phash": true,
            "enable_ssim": true,
            "enable_histogram": true,
            "enable_alpha_crop": false,
            "enable_rotations": false,
            "threshold": 0.85,
            "weights": { "phash": 0.35, "ssim": 0.45, "histogram": 0.2 }
        }"#;
        let cfg: Config = serde_json::from_str(json).unwrap();
        validate(&cfg).unwrap();
        assert_eq!(cfg.hash_algorithm, "sha256");
        assert_eq!(cfg.phash_max_distance, 10);
    }

    #[test]
    fn rejects_threshold_above_one() {
        let json = r#"{
            "enable_phash": true,
            "enable_ssim": true,
            "enable_histogram": true,
            "threshold": 1.1,
            "weights": { "phash": 0.35, "ssim": 0.45, "histogram": 0.2 }
        }"#;
        let cfg: Config = serde_json::from_str(json).unwrap();
        assert!(matches!(
            validate(&cfg),
            Err(ConfigError::ThresholdOutOfRange)
        ));
    }
}
