import type { ScanStatusState } from "$lib/types";

const BASE_TITLE = "Similar Textures";

export function buildWindowTitle(input: {
  running: boolean;
  statusState: ScanStatusState;
  resultGroupCount: number;
}): string {
  if (input.running) {
    return `${BASE_TITLE} — Scanning…`;
  }
  if (input.resultGroupCount > 0) {
    return `${BASE_TITLE} — ${input.resultGroupCount} groups`;
  }
  if (input.statusState === "failed") {
    return `${BASE_TITLE} — Failed`;
  }
  if (input.statusState === "cancelled") {
    return `${BASE_TITLE} — Cancelled`;
  }
  return BASE_TITLE;
}
