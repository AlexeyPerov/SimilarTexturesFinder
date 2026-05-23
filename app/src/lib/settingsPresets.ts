import type { AppSettings } from "$lib/types";
import { initialSettings } from "$lib/settingsValidation";

export type SettingsPresetId = "fast" | "balanced" | "strict";

export function applySettingsPreset(id: SettingsPresetId): AppSettings {
  switch (id) {
    case "fast":
      return {
        ...structuredClone(initialSettings),
        enable_phash: true,
        enable_ssim: false,
        enable_histogram: true,
        enable_alpha_crop: false,
        enable_rotations: false,
        enable_flip: false,
        threshold: 0.92,
        weights: { phash: 0.55, ssim: 0, histogram: 0.45, orb: 0 },
        max_decode_dimension_px: 512,
      };
    case "strict":
      return {
        ...structuredClone(initialSettings),
        enable_phash: true,
        enable_ssim: true,
        enable_histogram: true,
        enable_alpha_crop: true,
        enable_rotations: true,
        enable_flip: false,
        threshold: 0.78,
        weights: { phash: 0.35, ssim: 0.45, histogram: 0.2, orb: 0 },
        phash_max_distance: 12,
        ssim_threshold: 0.88,
      };
    case "balanced":
    default:
      return structuredClone(initialSettings);
  }
}
