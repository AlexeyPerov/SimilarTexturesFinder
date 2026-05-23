import type { ScanStatusState } from "$lib/types";

export type ProgressBarMode = "hidden" | "indeterminate" | "determinate";

export type ScanProgressUi = {
  phaseLabel: string;
  barMode: ProgressBarMode;
  processed?: number;
  total?: number;
};

const IDLE_PROGRESS: ScanProgressUi = {
  phaseLabel: "No active scan",
  barMode: "hidden",
};

export function statusDisplayLabel(state: ScanStatusState): string {
  switch (state) {
    case "idle":
      return "Ready";
    case "running":
      return "Scanning…";
    case "completed":
      return "Complete";
    case "failed":
      return "Failed";
    case "cancelled":
      return "Cancelled";
    default:
      return "Unknown";
  }
}

export function progressFromPhase(
  phase: string,
  processed?: number,
  total?: number,
  groups?: number,
): ScanProgressUi {
  switch (phase) {
    case "starting":
      return { phaseLabel: "Starting scan…", barMode: "indeterminate" };
    case "ingest_progress":
      return {
        phaseLabel: "Reading images",
        barMode: "determinate",
        processed,
        total,
      };
    case "ingest_done":
      return { phaseLabel: "Ingest complete", barMode: "indeterminate" };
    case "pairwise_start":
      return { phaseLabel: "Comparing images", barMode: "indeterminate" };
    case "clustering_done":
      return {
        phaseLabel: groups != null ? `Grouping results (${groups} groups)` : "Grouping results",
        barMode: "indeterminate",
      };
    case "scan_done":
      return { phaseLabel: "Finalizing", barMode: "indeterminate" };
    case "completed":
      return { phaseLabel: "Scan completed", barMode: "hidden" };
    case "cancelled":
      return { phaseLabel: "Scan cancelled", barMode: "hidden" };
    case "failed":
      return { phaseLabel: "Scan failed", barMode: "hidden" };
    default:
      return { phaseLabel: phase, barMode: "indeterminate" };
  }
}

export function idleProgress(): ScanProgressUi {
  return IDLE_PROGRESS;
}

export function formatElapsed(seconds: number): string {
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  return `${m}:${s.toString().padStart(2, "0")}`;
}
