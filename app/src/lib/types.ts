export type Weights = {
  phash: number;
  ssim: number;
  histogram: number;
  orb: number;
};

export type AppSettings = {
  enable_phash: boolean;
  enable_ssim: boolean;
  enable_histogram: boolean;
  enable_orb: boolean;
  enable_alpha_crop: boolean;
  enable_rotations: boolean;
  enable_flip: boolean;
  threshold: number;
  weights: Weights;
  hash_algorithm: string;
  phash_max_distance: number;
  ssim_threshold: number;
  resize_size: number;
  hist_bins: number;
  hist_method: string;
  alpha_threshold: number;
  orb_max_features: number | null;
  orb_match_threshold: number | null;
  max_decode_dimension_px: number | null;
  hide_single_image_groups: boolean;
};

export type SettingsValidationErrors = {
  threshold?: string;
  phashWeight?: string;
  ssimWeight?: string;
  histogramWeight?: string;
  phashMaxDistance?: string;
  ssimThreshold?: string;
  resizeSize?: string;
  histBins?: string;
  alphaThreshold?: string;
  maxDecodeDimension?: string;
};

export type GroupReasonKind = "singleton" | "hash" | "composite" | "mixed";

export type PairReasonType = "hash" | "composite";

export type MetricEvidence = {
  score: number;
  raw: number;
  valid: boolean;
};

export type GroupPairReason = {
  left: string;
  right: string;
  type: PairReasonType;
  composite_score?: number;
  phash?: MetricEvidence;
  ssim?: MetricEvidence;
  histogram?: MetricEvidence;
};

export type ScanGroup = {
  id: number;
  name?: string;
  score: number | null;
  images: string[];
  reason_kind: GroupReasonKind;
  reasons: GroupPairReason[];
};

export type ScanResult = {
  groups: ScanGroup[];
};

export type ScanStatusState = "idle" | "running" | "completed" | "failed" | "cancelled" | "unknown";

export type SortOption =
  | "count_desc"
  | "count_asc"
  | "name_asc"
  | "name_desc"
  | "score_desc"
  | "score_asc";

export type TabBanner = {
  kind: "success" | "error";
  message: string;
} | null;

export type GroupPreview = {
  id: number;
  title: string;
  scoreLabel: string;
  images: string[];
  hiddenCount: number;
  count: number;
  reasonKind: GroupReasonKind;
};

export type UiStateResponse = {
  lastInputDir: string | null;
};
