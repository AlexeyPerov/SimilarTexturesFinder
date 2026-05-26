import type { AppSettings } from "$lib/types";
import { detectSettingsPreset, formatSettingsSummary } from "$lib/settingsSummary";

/** Rough per-image ingest cost in milliseconds (feature extraction always runs for all metrics). */
const BASE_INGEST_MS = 18;

/** Additional ingest cost multiplier when alpha crop is enabled. */
const ALPHA_CROP_MULT = 1.12;

/** Decode cost scale when max dimension is unset (full resolution). */
const FULL_DECODE_MULT = 2.4;

/** Decode cost scale reference at 512px cap. */
const DECODE_REF_PX = 512;

/** Base pairwise comparison cost in microseconds per pair (pHash only). */
const BASE_PAIR_US = 45;

/** Extra pairwise cost multipliers per enabled metric. */
const SSIM_PAIR_MULT = 4.5;
const HISTOGRAM_PAIR_MULT = 1.8;

/** Clustering overhead per image in milliseconds. */
const CLUSTER_MS_PER_IMAGE = 0.08;

/** Range factors applied to the point estimate. */
const LOW_FACTOR = 0.6;
const HIGH_FACTOR = 1.8;

export type ScanTimeEstimate = {
  lowSeconds: number;
  highSeconds: number;
  lines: string[];
};

function transformVariantCount(settings: AppSettings): number {
  if (!settings.enable_rotations && !settings.enable_flip) return 1;
  if (settings.enable_rotations && settings.enable_flip) return 8;
  if (settings.enable_rotations) return 4;
  return 2;
}

function decodeMultiplier(settings: AppSettings): number {
  const cap = settings.max_decode_dimension_px;
  if (cap == null) return FULL_DECODE_MULT;
  return Math.max(0.5, cap / DECODE_REF_PX);
}

function ingestMsPerImage(settings: AppSettings): number {
  let ms = BASE_INGEST_MS * decodeMultiplier(settings) * transformVariantCount(settings);
  if (settings.enable_alpha_crop) ms *= ALPHA_CROP_MULT;
  return ms;
}

function pairCostMultiplier(settings: AppSettings): number {
  let mult = 0;
  if (settings.enable_phash) mult += 1;
  if (settings.enable_histogram) mult += HISTOGRAM_PAIR_MULT;
  if (settings.enable_ssim) mult += SSIM_PAIR_MULT;
  return Math.max(mult, 0.5);
}

function pairLoopFactor(settings: AppSettings): number {
  const variants = transformVariantCount(settings);
  if (variants <= 1) return 1;
  return 1 + (variants - 1) * 0.35;
}

function formatDuration(seconds: number): string {
  if (seconds < 60) return `~${Math.max(1, Math.round(seconds))} sec`;
  const mins = Math.round(seconds / 60);
  if (mins < 120) return `~${mins} min`;
  const hours = Math.floor(mins / 60);
  const rem = mins % 60;
  return rem > 0 ? `~${hours}h ${rem}m` : `~${hours}h`;
}

function formatDurationRange(lowSeconds: number, highSeconds: number): string {
  if (highSeconds < 60) {
    return `~${Math.max(1, Math.round(lowSeconds))}–${Math.max(1, Math.round(highSeconds))} sec`;
  }
  const lowMin = Math.max(1, Math.round(lowSeconds / 60));
  const highMin = Math.max(lowMin, Math.round(highSeconds / 60));
  return `~${lowMin}–${highMin} min`;
}

function formatPairCount(n: number): string {
  return n.toLocaleString("en-US");
}

export function estimateScanTime(
  imageCount: number,
  settings: AppSettings,
  threads: number,
  inputDir: string,
): ScanTimeEstimate {
  const n = imageCount;
  const threadCount = Math.max(1, threads);
  const pairs = n >= 2 ? (n * (n - 1)) / 2 : 0;

  const ingestSec = (n * ingestMsPerImage(settings)) / 1000;
  const pairUs =
    pairs * BASE_PAIR_US * pairCostMultiplier(settings) * pairLoopFactor(settings);
  const pairSec = pairUs / 1_000_000 / threadCount;
  const clusterSec = (n * CLUSTER_MS_PER_IMAGE) / 1000;

  const pointSeconds = ingestSec + pairSec + clusterSec;
  const lowSeconds = pointSeconds * LOW_FACTOR;
  const highSeconds = pointSeconds * HIGH_FACTOR;

  const preset = detectSettingsPreset(settings);
  const presetLabel = preset === "custom" ? "Custom" : preset.charAt(0).toUpperCase() + preset.slice(1);
  const analysisSummary = formatSettingsSummary(settings);

  const lines = [
    "Time estimate (heuristic):",
    `  Folder: ${inputDir}`,
    `  Images: ${n.toLocaleString("en-US")}`,
    `  Threads: ${threadCount}`,
    `  Analysis: ${presetLabel} · ${analysisSummary.split(" · ").slice(1).join(" · ")}`,
    `  Pair comparisons (worst case): ~${formatPairCount(pairs)}`,
    `  Estimated duration: ${formatDurationRange(lowSeconds, highSeconds)} (midpoint ${formatDuration(pointSeconds)})`,
    "  Assumes few byte-identical files; actual time varies by resolution, format, and disk speed.",
  ];

  return { lowSeconds, highSeconds, lines };
}

export function defaultThreadCount(): number {
  if (typeof navigator !== "undefined" && navigator.hardwareConcurrency) {
    return navigator.hardwareConcurrency;
  }
  return 4;
}
