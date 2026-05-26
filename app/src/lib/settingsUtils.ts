import type { AppSettings } from "$lib/types";

export function cloneAppSettings(settings: AppSettings): AppSettings {
  return JSON.parse(JSON.stringify(settings)) as AppSettings;
}

export function settingsFingerprint(settings: AppSettings): string {
  return JSON.stringify(settings);
}

export function settingsEqual(a: AppSettings, b: AppSettings): boolean {
  return settingsFingerprint(a) === settingsFingerprint(b);
}
