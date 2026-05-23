import type { AppSettings, SettingsValidationErrors } from "$lib/types";

export function validateSettings(next: AppSettings): SettingsValidationErrors {
  const errors: SettingsValidationErrors = {};
  if (next.threshold < 0 || next.threshold > 1) {
    errors.threshold = "Threshold must be between 0 and 1.";
  }
  if (next.weights.phash < 0) {
    errors.phashWeight = "pHash weight cannot be negative.";
  }
  if (next.weights.ssim < 0) {
    errors.ssimWeight = "SSIM weight cannot be negative.";
  }
  if (next.weights.histogram < 0) {
    errors.histogramWeight = "Histogram weight cannot be negative.";
  }
  if (next.phash_max_distance < 0) {
    errors.phashMaxDistance = "pHash max distance cannot be negative.";
  }
  if (next.ssim_threshold < 0 || next.ssim_threshold > 1) {
    errors.ssimThreshold = "SSIM threshold must be between 0 and 1.";
  }
  if (next.resize_size < 1) {
    errors.resizeSize = "Resize size must be at least 1.";
  }
  if (next.hist_bins < 2) {
    errors.histBins = "Histogram bins must be at least 2.";
  }
  if (next.alpha_threshold < 0 || next.alpha_threshold > 1) {
    errors.alphaThreshold = "Alpha threshold must be between 0 and 1.";
  }
  if (next.max_decode_dimension_px != null && next.max_decode_dimension_px < 1) {
    errors.maxDecodeDimension = "Max decode dimension must be at least 1.";
  }
  if (!next.enable_phash && !next.enable_ssim && !next.enable_histogram) {
    errors.threshold = "Enable at least one metric (pHash, SSIM, or histogram).";
  }
  return errors;
}

export const initialSettings: AppSettings = {
  enable_phash: true,
  enable_ssim: true,
  enable_histogram: true,
  enable_orb: false,
  enable_alpha_crop: false,
  enable_rotations: false,
  enable_flip: false,
  threshold: 0.85,
  weights: { phash: 0.35, ssim: 0.45, histogram: 0.2, orb: 0 },
  hash_algorithm: "sha256",
  phash_max_distance: 10,
  ssim_threshold: 0.9,
  resize_size: 256,
  hist_bins: 512,
  hist_method: "correlation",
  alpha_threshold: 0.05,
  orb_max_features: null,
  orb_match_threshold: null,
  max_decode_dimension_px: null,
  hide_single_image_groups: true,
};
