export const settingsHelp = {
  enable_phash: "Perceptual hash — fast visual fingerprint for near-duplicate detection.",
  enable_ssim: "Structural similarity — compares luminance patterns after resize.",
  enable_histogram: "Color histogram comparison — useful for palette-level similarity.",
  enable_alpha_crop: "Trim transparent borders before comparing textures with alpha.",
  enable_rotations: "Also compare rotated variants (slower, more thorough).",
  enable_flip: "Also compare horizontally flipped variants.",
  hide_single_image_groups: "Hide groups that contain only one image in Results.",
  threshold: "Minimum composite similarity score (0–1) to link two images.",
  phash_weight: "Weight of pHash in the composite score.",
  ssim_weight: "Weight of SSIM in the composite score.",
  histogram_weight: "Weight of histogram similarity in the composite score.",
  hash_algorithm: "Algorithm used for byte-identical file hashing (duplicate detection).",
  phash_max_distance: "Maximum Hamming distance for pHash pairs to be considered similar.",
  ssim_threshold: "Minimum raw SSIM value before it contributes to matching.",
  resize_size: "Side length used when resizing images for metric extraction.",
  hist_bins: "Number of histogram bins per channel.",
  hist_method: "Method used to compare color histograms.",
  alpha_threshold: "Alpha values below this are treated as transparent when cropping.",
  max_decode_dimension: "Downscale large images on decode to save memory (optional).",
  orb_note: "ORB feature matching is not available in this build.",
} as const;

export const reasonLegendEntries = [
  { kind: "singleton", label: "Singleton", description: "Only one image in this group." },
  { kind: "hash", label: "Hash", description: "Files are byte-identical." },
  { kind: "composite", label: "Composite", description: "Visually similar by combined metrics." },
  { kind: "mixed", label: "Mixed", description: "Multiple match types in one group." },
] as const;

export const metricDetailsHelp = {
  metric_details:
    "Technical breakdown of how this image pair was scored by each similarity metric.",
  composite:
    "Weighted blend of enabled metrics that passed their checks. Range 0–1; higher means more similar.",
  phash: "Perceptual hash — fast visual fingerprint of overall image structure.",
  ssim: "Structural similarity — compares luminance patterns after both images are resized.",
  histogram: "Color histogram comparison — measures how alike the color distributions are.",
  score: "Normalized similarity used for matching, on a 0–1 scale (higher = more similar).",
  phash_raw: "Hamming distance between 64-bit hashes (0 = identical; lower is more similar).",
  ssim_raw: "Raw SSIM value from the luminance comparison (0–1 scale).",
  histogram_raw: "Raw histogram comparison value (correlation or Bhattacharyya coefficient).",
  valid:
    "Whether this metric met its threshold and was included in the composite score.",
} as const;
