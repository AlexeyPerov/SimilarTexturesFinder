import type { AppSettings } from "$lib/types";

export function settingsFingerprint(settings: AppSettings): string {
  return JSON.stringify(settings);
}

export function settingsEqual(a: AppSettings, b: AppSettings): boolean {
  return settingsFingerprint(a) === settingsFingerprint(b);
}
