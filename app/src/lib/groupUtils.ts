import type { AppSettings, GroupPreview, GroupReasonKind, ScanGroup } from "$lib/types";

export const maxResultThumbs = 10;
export const maxGroupsPerPage = 15;
export const reasonKindOrder: GroupReasonKind[] = ["singleton", "hash", "composite", "mixed"];

export function toBaseName(path: string): string {
  return path.split(/[\\/]/).at(-1) ?? path;
}

export function groupTitle(group: Pick<ScanGroup, "id" | "name">): string {
  const explicit = group.name?.trim();
  return explicit && explicit.length > 0 ? explicit : `Group #${group.id}`;
}

export function reasonKindLabel(kind: GroupReasonKind): string {
  switch (kind) {
    case "singleton":
      return "Singleton";
    case "hash":
      return "Hash";
    case "composite":
      return "Composite";
    case "mixed":
      return "Mixed";
    default:
      return kind;
  }
}

export function pairReasonTypeLabel(reasonType: "hash" | "composite"): string {
  switch (reasonType) {
    case "hash":
      return "Hash";
    case "composite":
      return "Composite";
    default:
      return reasonType;
  }
}

export function formatMetricValue(value: number): string {
  return Number.isFinite(value) ? value.toFixed(3) : String(value);
}

export function compareGroupNames(a: ScanGroup, b: ScanGroup): number {
  const nameA = a.name?.trim();
  const nameB = b.name?.trim();
  if (nameA && nameB) {
    const byName = nameA.localeCompare(nameB, undefined, { sensitivity: "base" });
    return byName !== 0 ? byName : a.id - b.id;
  }
  if (nameA && !nameB) return -1;
  if (!nameA && nameB) return 1;
  return a.id - b.id;
}

export function groupedPreview(groups: ScanGroup[]): GroupPreview[] {
  return groups.map((g) => ({
    id: g.id,
    title: groupTitle(g),
    scoreLabel: g.score == null ? "-" : g.score.toFixed(3),
    images: g.images.slice(0, maxResultThumbs),
    hiddenCount: Math.max(0, g.images.length - maxResultThumbs),
    count: g.images.length,
    reasonKind: g.reason_kind,
  }));
}

export function filterVisibleGroups(groups: ScanGroup[], settings: AppSettings): ScanGroup[] {
  return groups.filter(
    (group) => !settings.hide_single_image_groups || group.images.length > 1,
  );
}

export function sortGroups(groups: ScanGroup[], sortOption: string): ScanGroup[] {
  const sorted = [...groups];
  switch (sortOption) {
    case "count_asc":
      sorted.sort((a, b) => a.images.length - b.images.length || a.id - b.id);
      break;
    case "name_asc":
      sorted.sort(compareGroupNames);
      break;
    case "name_desc":
      sorted.sort((a, b) => compareGroupNames(b, a));
      break;
    case "count_desc":
    default:
      sorted.sort((a, b) => b.images.length - a.images.length || a.id - b.id);
      break;
  }
  return sorted;
}
