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

export function filterGroupsBySearch(groups: ScanGroup[], query: string): ScanGroup[] {
  const needle = query.trim().toLowerCase();
  if (!needle) return groups;
  return groups.filter((group) =>
    group.images.some((path) => {
      const base = toBaseName(path).toLowerCase();
      return path.toLowerCase().includes(needle) || base.includes(needle);
    }),
  );
}

export function countUniqueImages(groups: ScanGroup[]): number {
  const seen = new Set<string>();
  for (const group of groups) {
    for (const path of group.images) {
      seen.add(path);
    }
  }
  return seen.size;
}

function compareScores(a: ScanGroup, b: ScanGroup, ascending: boolean): number {
  const scoreA = a.score;
  const scoreB = b.score;
  if (scoreA == null && scoreB == null) return a.id - b.id;
  if (scoreA == null) return 1;
  if (scoreB == null) return -1;
  const byScore = ascending ? scoreA - scoreB : scoreB - scoreA;
  return byScore !== 0 ? byScore : a.id - b.id;
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
    case "score_asc":
      sorted.sort((a, b) => compareScores(a, b, true));
      break;
    case "score_desc":
      sorted.sort((a, b) => compareScores(a, b, false));
      break;
    case "count_desc":
    default:
      sorted.sort((a, b) => b.images.length - a.images.length || a.id - b.id);
      break;
  }
  return sorted;
}
