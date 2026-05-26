import type { AppSettings } from "$lib/types";
import {
  applySettingsPreset,
  type SettingsPresetId,
} from "$lib/settingsPresets";
import { settingsEqual } from "$lib/settingsUtils";

const PRESET_IDS: SettingsPresetId[] = ["fast", "balanced", "strict"];

export function detectSettingsPreset(settings: AppSettings): SettingsPresetId | "custom" {
  for (const id of PRESET_IDS) {
    if (settingsEqual(settings, applySettingsPreset(id))) {
      return id;
    }
  }
  return "custom";
}

function formatEnabledMetrics(settings: AppSettings): string {
  const metrics: string[] = [];
  if (settings.enable_phash) metrics.push("pHash");
  if (settings.enable_ssim) metrics.push("SSIM");
  if (settings.enable_histogram) metrics.push("histogram");
  return metrics.length > 0 ? metrics.join(", ") : "none";
}

function formatTransformFlags(settings: AppSettings): string {
  const parts: string[] = [];
  if (settings.enable_rotations) parts.push("rotations");
  if (settings.enable_flip) parts.push("flip");
  if (settings.enable_alpha_crop) parts.push("alpha crop");
  return parts.length > 0 ? parts.join(", ") : "no transforms";
}

function formatDecodeCap(settings: AppSettings): string {
  if (settings.max_decode_dimension_px == null) {
    return "full decode";
  }
  return `decode max ${settings.max_decode_dimension_px}px`;
}

function capitalizePreset(id: SettingsPresetId | "custom"): string {
  if (id === "custom") return "Custom";
  return id.charAt(0).toUpperCase() + id.slice(1);
}

export function formatSettingsSummary(settings: AppSettings): string {
  const preset = detectSettingsPreset(settings);
  const parts = [
    capitalizePreset(preset),
    formatEnabledMetrics(settings),
    `threshold ${settings.threshold}`,
    formatTransformFlags(settings),
    formatDecodeCap(settings),
  ];
  return parts.join(" · ");
}
