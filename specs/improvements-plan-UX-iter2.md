# UX Improvements Plan — Iteration 2

## Overview

This document describes a phased plan to improve the **Similar Textures** desktop app (Tauri 2 + Svelte 5) from a user-experience perspective. It complements `specs/improvements-plan.md`, which focuses on backend stability and performance.

Goals:

- Make scans feel **responsive and trustworthy** (progress, feedback, errors).
- Reduce friction in the **Scan → Results** loop (folder selection, re-run, persistence).
- Turn Results from a passive viewer into a **useful duplicate-finding tool** (actions, search, comparison).
- Lower the cognitive load of **Settings** (grouping, presets, stale-result hints).

Each phase is independently shippable. Phases are ordered by user impact; agents may parallelize tasks within a phase when dependencies allow.

---

## Current baseline (as of iter 2 planning)

| Area | Current behavior |
|------|------------------|
| UI surface | Single file: `app/src/routes/+page.svelte` (~1,860 lines) |
| Tabs | `Scan` and `Results` only; Settings is a modal |
| Input folder | In-memory only; not persisted across sessions |
| Browse control | Button labeled `...` |
| Scan progress | Two text pills (`statusText`, `progressText`); ingest shows `processed/total` |
| Backend progress events | Rich phases via `scan-progress` (`ingest_progress`, `pairwise_start`, `clustering_done`, …) — mostly underused in UI |
| Results actions | Export JSON only; `tauri-plugin-opener` installed but unused in UI |
| Results data | In-memory only; lost on app restart |
| Thumbnails | Full-resolution via `convertFileSrc`; 76px display tiles |
| Settings | Flat grid of ~15 controls; no presets or grouping |
| ORB | Present in `Config` / TS types; no settings UI |
| Accessibility | Partial (tab roles, some aria-labels); overlay backdrop uses `role="button"` |

Reference README behavior: `specs/` and repo root `README.md`.

---

## How agents should use this document

1. **Pick one milestone item** (e.g. UX-1.2) per PR unless the item explicitly says otherwise.
2. **Read dependencies** — do not start an item until its `Depends on` list is satisfied.
3. **Scope tightly** — match existing dark-theme styling in `+page.svelte`; avoid unrelated refactors.
4. **Prefer extraction when touching UI repeatedly** — Phase UX-0 is optional but recommended before large Phase UX-2/UX-3 work.
5. **Verify manually** after each item (see per-item Verification blocks).
6. **Update status** in the Execution tracker at the bottom when completing work.

### Checks to run

```bash
# Frontend
cd app && npm run check

# Rust host (if backend touched)
cd app/src-tauri && cargo test
```

### Key files

| File | Role |
|------|------|
| `app/src/routes/+page.svelte` | All UI today |
| `app/src/lib/logSegments.ts` | Console error segmentation |
| `app/src-tauri/src/lib.rs` | Tauri commands, events, settings persistence |
| `app/src-tauri/crates/similar-textures-core/src/config.rs` | Scan settings schema |
| `app/src-tauri/crates/similar-textures-core/src/lib.rs` | `ScanEvent` emission |

---

## Phase UX-0 — Component extraction (enabler, optional first)

Splitting the monolith makes later UX work safer and enables targeted performance fixes (virtualization, scoped re-renders).

### UX-0.1 Extract presentational components

**Files:** new under `app/src/lib/components/`, update `+page.svelte`

| Component | Responsibility |
|-----------|----------------|
| `ScanPanel.svelte` | Folder input, actions, console |
| `ResultsPanel.svelte` | Toolbar, grid, empty states |
| `SettingsModal.svelte` | Settings form + validation |
| `GroupDetailModal.svelte` | Group images + pair reasons |
| `ProgressBar.svelte` | Reusable determinate/indeterminate bar |

**Depends on:** none

**Verification:**

- [ ] App behavior unchanged (manual smoke: scan, results, settings, group detail).
- [ ] `npm run check` passes.

---

## Phase UX-1 — Scan workflow & feedback (highest impact)

Users must trust that long scans are progressing and understand what went wrong when they fail.

### UX-1.1 Human-readable status labels

**File:** `app/src/routes/+page.svelte` (or `ScanPanel.svelte`)

Replace raw status strings (`idle`, `running`, `completed`, …) with user-facing copy:

| Internal | Display |
|----------|---------|
| `idle` | Ready |
| `running` | Scanning… |
| `completed` | Complete |
| `failed` | Failed |
| `cancelled` | Cancelled |

**Depends on:** none

**Verification:**

- [ ] Status pill shows friendly text during and after scan.

### UX-1.2 Progress bar + phase labels

**Files:** `+page.svelte`, optionally `ProgressBar.svelte`, `lib.rs` (only if new event fields needed)

Map `scan-progress` phases to UI:

| Phase | UI label | Bar mode |
|-------|----------|----------|
| `ingest_progress` | Reading images | Determinate (`processed / total`) |
| `ingest_done` | Ingest complete | Indeterminate or brief transition |
| `pairwise_start` | Comparing images | Indeterminate (no pair-level events yet) |
| `clustering_done` | Grouping results | Indeterminate |
| `scan_done` | Finalizing | Indeterminate |

Show **elapsed time** since scan start (update every second while `running`).

Optional follow-up (separate item): emit pairwise progress from core (`ScanEvent::PairwiseProgress`) for determinate bar in pairwise phase.

**Depends on:** none (UX-0.1 optional)

**Verification:**

- [ ] During ingest, bar reflects `processed/total`.
- [ ] Phase label updates through a full scan on `test-images/`.
- [ ] Elapsed timer visible while running; stops on finish.

### UX-1.3 Inline validation for empty input folder

**File:** `+page.svelte`

When Run Scan is clicked with empty `inputDir`:

- Show inline error under the path field (not only a log line).
- Do not clear logs or change `running` state.

**Depends on:** none

**Verification:**

- [ ] Empty folder → visible inline message; no scan started.

### UX-1.4 Improve folder picker affordance

**File:** `+page.svelte`

Replace `...` browse button with labeled **Browse…** or a folder icon + accessible name.

Optional: drag-and-drop folder onto scan panel (Tauri/webview drop APIs).

**Depends on:** none

**Verification:**

- [ ] Button purpose is obvious without hover tooltip.
- [ ] If DnD implemented: dropping a folder sets `inputDir`.

### UX-1.5 Persist last input folder

**Files:** `app/src-tauri/src/lib.rs`, `+page.svelte`

Persist `lastInputDir` in app config (extend `settings.json` or separate `ui-state.json` with schema version).

- Load on mount; pre-fill path field.
- Save when scan starts successfully (or when folder chosen — pick one, document in PR).

**Depends on:** none

**Verification:**

- [ ] Restart app → last folder still shown.
- [ ] Invalid/missing path on disk: field shows saved value; scan fails gracefully with clear message.

### UX-1.6 Toast / banner feedback for scan-adjacent actions

**File:** `+page.svelte` (+ small `Toast.svelte` or inline banner state)

Surface success/error on the **active tab**:

| Action | Feedback |
|--------|----------|
| Export JSON | Success: path shown on Results; error: banner |
| Copy logs | Success: brief “Copied”; failure: banner |
| Save settings | Keep existing modal message or toast after close |

Avoid routing export success only to Scan logs.

**Depends on:** none

**Verification:**

- [ ] Export from Results shows confirmation on Results tab.
- [ ] Copy logs shows confirmation.

### UX-1.7 Scan failure visibility

**File:** `+page.svelte`

On `scan-finished` with `failed`:

- Do **not** auto-switch to Results.
- Show prominent error banner on Scan (use `message` from payload if present).
- Optionally scroll console to latest error segment.

**Depends on:** UX-1.6 (banner pattern)

**Verification:**

- [ ] Induced failure → user stays on Scan with visible error.

---

## Phase UX-2 — Results exploration & actions

Turn Results into a tool for finding and acting on duplicate textures.

### UX-2.1 Thumbnail decode limits in UI

**Files:** `+page.svelte`, possibly new Tauri command for resized preview

Problem: `convertFileSrc` loads full-resolution images into 76px tiles.

Options (pick one in implementation PR):

1. **Frontend-only:** rely on `max_decode_dimension_px` setting + document that users should set it for large libraries.
2. **Backend thumbnail command:** `get_thumbnail(path, maxPx) -> temp file or bytes` used by `<img src>`.

Target: grid scroll stays smooth with 500+ groups (15 per page × up to 10 thumbs).

**Depends on:** none

**Verification:**

- [ ] Large PNG/JPEG textures in grid do not cause obvious UI jank.
- [ ] Group detail modal uses same limited preview path.

### UX-2.2 Search by filename / path

**File:** `ResultsPanel.svelte` or `+page.svelte`

Add search input filtering groups where **any** image path or basename matches (case-insensitive).

Reset pagination to page 1 on search change.

**Depends on:** none

**Verification:**

- [ ] Typing filters groups live.
- [ ] Empty search restores full list.
- [ ] “No groups match” empty state when filter yields zero groups.

### UX-2.3 Sort by similarity score

**File:** `+page.svelte`

Add sort options:

- Score (high to low)
- Score (low to high)

Groups with `score == null` sort last (document behavior).

**Depends on:** none

**Verification:**

- [ ] Sort changes order correctly on fixture with varied scores.

### UX-2.4 Quick toggle: hide single-image groups

**File:** `+page.svelte`

Expose `hide_single_image_groups` as a checkbox on Results toolbar (two-way bound to persisted settings, or local override with “Save to settings” — prefer direct bind + save on toggle or mirror settings draft).

Avoid forcing users into Settings modal for a common filter.

**Depends on:** none

**Verification:**

- [ ] Toggle updates visible groups without rescan.
- [ ] Setting persists across restart if wired to settings.

### UX-2.5 Reveal in Finder / Explorer

**Files:** `+page.svelte`, `@tauri-apps/plugin-opener`

Use `revealItemInDir` (or platform equivalent) from:

- Group detail: per-image action (context menu or icon on thumbnail).
- Optional: group-level “Reveal folder” if all images share a directory.

**Depends on:** none

**Verification:**

- [ ] macOS: Finder reveals file; Windows: Explorer selects file (when tested).

### UX-2.6 Copy paths

**File:** `+page.svelte`

Actions:

- Copy single image path (thumbnail click or button).
- Copy all paths in group (button in group detail header).

Use clipboard API with success toast (UX-1.6).

**Depends on:** UX-1.6

**Verification:**

- [ ] Paste yields correct absolute paths.

### UX-2.7 Import / load exported JSON

**Files:** `lib.rs` (new command e.g. `load_result_json`), `+page.svelte`

Allow user to open a previously exported `result.json` into in-memory results (file picker).

- Validate schema; show error banner on parse failure.
- Do not overwrite settings.

**Depends on:** none

**Verification:**

- [ ] Export → restart app → Import → Results match prior export.

### UX-2.8 Results summary strip

**File:** `+page.svelte`

When `result` present, show summary near header:

- Group count (after filters)
- Total image count (unique paths across visible groups, or total in result — document choice)
- Optional: last scan duration if tracked client-side

**Depends on:** UX-1.2 (duration tracking) optional

**Verification:**

- [ ] Summary updates when filters change.

### UX-2.9 Results tab badge + Re-run scan

**File:** `+page.svelte`

- Show group count badge on Results tab when `result` exists (e.g. `Results (42)`).
- Add **Scan again** on Results using current `inputDir` (disabled if empty or scan running).

**Depends on:** UX-1.5 (folder persist) recommended

**Verification:**

- [ ] Badge visible after successful scan.
- [ ] Scan again starts scan without switching tabs manually first.

### UX-2.10 Pair comparison in group detail (stretch)

**File:** `GroupDetailModal.svelte`

For each pair reason, show two thumbnails side-by-side with similarity highlight (composite score as bar or color).

Collapse raw metric chips behind “Details” disclosure for advanced users.

**Depends on:** UX-2.1 (thumbnail limits)

**Verification:**

- [ ] Hash and composite pairs visually comparable without reading raw numbers.

---

## Phase UX-3 — Settings simplification

Reduce intimidation for non-expert users while keeping full control for power users.

### UX-3.1 Group settings into sections

**File:** `SettingsModal.svelte` or `+page.svelte`

Sections:

1. **Matching** — enable pHash / SSIM / histogram, threshold, hide single-image groups
2. **Transforms** — alpha crop, rotations, flip
3. **Weights & thresholds** — weights, pHash max distance, SSIM threshold
4. **Performance** — resize size, hist bins/method, max decode dimension
5. **Advanced** — hash algorithm

Use collapsible `<details>` or subheadings consistent with existing dark theme.

**Depends on:** none (UX-0.1 optional)

**Verification:**

- [ ] All existing fields still reachable.
- [ ] Validation unchanged.

### UX-3.2 Presets: Fast / Balanced / Strict

**File:** `SettingsModal.svelte`

Buttons apply predefined `AppSettings` drafts (document values in PR):

| Preset | Intent |
|--------|--------|
| Fast | Fewer metrics, higher threshold, no rotations |
| Balanced | Current defaults |
| Strict | Lower threshold, all metrics, rotations optional |

User must still click **Save** to persist.

**Depends on:** UX-3.1

**Verification:**

- [ ] Each preset loads plausible values passing validation.

### UX-3.3 Field help tooltips

**File:** `SettingsModal.svelte`

Short `title` or info icon popover for: pHash, SSIM, histogram, alpha crop, reason kinds (cross-link to Results legend if UX-3.5 done).

**Depends on:** UX-3.1

**Verification:**

- [ ] Every non-obvious field has one sentence of help.

### UX-3.4 Stale results hint

**File:** `+page.svelte`

Track hash or timestamp of settings used for last scan (return from `start_scan` or store client-side snapshot on scan start).

When current saved settings differ from last-scan settings and `result` exists, show banner on Results:

> Settings changed since last scan — run Scan again to apply.

**Depends on:** none

**Verification:**

- [ ] Change threshold → save → Results shows stale hint until rescan.

### UX-3.5 Unsaved settings guard

**File:** `SettingsModal.svelte`

If `settingsDraft` differs from `settings` when closing modal (backdrop, Escape, X):

- Confirm dialog: Discard changes / Keep editing.

**Depends on:** none

**Verification:**

- [ ] Editing field then clicking backdrop prompts confirm.

### UX-3.6 ORB settings exposure (optional / blocked)

**Files:** `config.rs`, `SettingsModal.svelte`

Only implement when ORB is active in core (`enable_orb` not `dead_code`):

- Checkbox + weight + orb_max_features + orb_match_threshold

Until then, add spec note in UI: “ORB matching not available in this build.”

**Depends on:** core ORB implementation

**Verification:**

- [ ] N/A until core supports ORB in scan pipeline.

---

## Phase UX-4 — Reason kinds & copy (clarity)

### UX-4.1 Results legend for reason badges

**File:** `+page.svelte`

Compact legend or `?` popover near reason filter chips:

| Kind | User-facing explanation |
|------|---------------------------|
| Singleton | Only one image in group |
| Hash | Files are byte-identical |
| Composite | Visually similar by combined metrics |
| Mixed | Multiple match types in one group |

**Depends on:** none

**Verification:**

- [ ] Legend visible on Results when groups exist.

### UX-4.2 Softer metric copy in group detail

**File:** `GroupDetailModal.svelte`

Replace dense single-line metrics with:

- Primary: pair names + type + composite score
- Secondary (collapsed): pHash / SSIM / histogram raw details

**Depends on:** UX-2.10 optional overlap

**Verification:**

- [ ] Non-expert readable at a glance.

---

## Phase UX-5 — Desktop polish & accessibility

### UX-5.1 Keyboard shortcuts

**File:** `+page.svelte`

| Shortcut | Action |
|----------|--------|
| `Cmd/Ctrl+,` | Open settings |
| `Enter` (path focused) | Start scan if idle |
| `Escape` | Close topmost modal |

**Depends on:** none

**Verification:**

- [ ] Shortcuts work on macOS; document Windows equivalents.

### UX-5.2 Open image on double-click

**File:** `+page.svelte`, `plugin-opener`

Double-click thumbnail → open file in default viewer (`openPath`).

**Depends on:** UX-2.5

**Verification:**

- [ ] Image opens in system viewer.

### UX-5.3 Modal focus trap & backdrop fix

**File:** modal components

- Trap focus inside modal while open.
- Change backdrop from `role="button"` to `role="presentation"` with separate click handler; ensure close button and Escape still work.

**Depends on:** UX-0.1 optional

**Verification:**

- [ ] Tab cycles within modal only.
- [ ] Screen reader announces dialog title.

### UX-5.4 Window title / document subtitle

**Files:** `+page.svelte`, optionally `tauri.conf.json`

Reflect state in title: e.g. `Similar Textures — Scanning…` / `Similar Textures — 42 groups`.

Use Tauri window API if needed.

**Depends on:** UX-1.1, UX-2.9

**Verification:**

- [ ] Title updates during scan and after results load.

---

## Phase UX-6 — Export formats (stretch)

### UX-6.1 Export CSV duplicate report

**Files:** `lib.rs`, `+page.svelte`

CSV columns: `group_id`, `group_name`, `score`, `reason_kind`, `image_path`

Button next to Export JSON.

**Depends on:** none

**Verification:**

- [ ] CSV opens in spreadsheet app with expected rows.

---

## Priority order (recommended execution sequence)

| Order | Item | Rationale |
|-------|------|-----------|
| 1 | UX-1.2, UX-1.1 | Trust during long scans |
| 2 | UX-1.3, UX-1.4, UX-1.5 | Reduce scan friction |
| 3 | UX-1.6, UX-1.7 | Consistent feedback |
| 4 | UX-2.1 | Performance perception |
| 5 | UX-2.2, UX-2.3, UX-2.4 | Results usability |
| 6 | UX-2.5, UX-2.6, UX-2.7 | Actionable output |
| 7 | UX-2.8, UX-2.9 | Loop closure |
| 8 | UX-3.1–UX-3.5 | Settings accessibility |
| 9 | UX-4.1, UX-4.2 | Terminology clarity |
| 10 | UX-5.x | Polish |
| 11 | UX-0.1 | Do earlier if multiple agents touch UI in parallel |
| 12 | UX-2.10, UX-6.1 | Stretch |

---

## Out of scope for this iteration

- Auto-rescan when settings change (explicit non-goal per README).
- Migration from legacy Flutter UI or data.
- Windows packaging/signing polish.
- Backend performance work (see `specs/improvements-plan.md`).
- Duplicate file deletion / quarantine (destructive actions need explicit product decision).

---

## Acceptance criteria (iteration complete)

When Phases UX-1 through UX-4 are done:

- [ ] A first-time user can pick a folder, run a scan, and see **visual progress** without reading the console.
- [ ] Last folder persists across app restarts.
- [ ] Export and copy actions show **on-tab confirmation**, not hidden logs.
- [ ] Results support **search**, **score sort**, and **hide singletons** without opening Settings.
- [ ] User can **reveal** and **copy** image paths from Results.
- [ ] User can **import** a prior JSON export after restart.
- [ ] Settings are **grouped** with at least one **preset**; stale-results hint appears after settings change.
- [ ] Reason badges have a **legend**; group detail is readable without metric expertise.
- [ ] `npm run check` and `cargo test` pass.

---

## Execution tracker

Mark status: `[ ]` not started · `[~]` in progress · `[x]` done

| ID | Title | Status | Depends on | PR |
|----|-------|--------|------------|-----|
| UX-0.1 | Extract components | [ ] | — | |
| UX-1.1 | Human-readable status | [ ] | — | |
| UX-1.2 | Progress bar + phases + elapsed | [ ] | — | |
| UX-1.3 | Inline empty-folder validation | [ ] | — | |
| UX-1.4 | Browse / DnD folder | [ ] | — | |
| UX-1.5 | Persist last folder | [ ] | — | |
| UX-1.6 | Toasts / banners | [ ] | — | |
| UX-1.7 | Scan failure visibility | [ ] | UX-1.6 | |
| UX-2.1 | Thumbnail decode limits | [ ] | — | |
| UX-2.2 | Search by filename | [ ] | — | |
| UX-2.3 | Sort by score | [ ] | — | |
| UX-2.4 | Hide singletons toggle on Results | [ ] | — | |
| UX-2.5 | Reveal in Finder/Explorer | [ ] | — | |
| UX-2.6 | Copy paths | [ ] | UX-1.6 | |
| UX-2.7 | Import JSON results | [ ] | — | |
| UX-2.8 | Results summary strip | [ ] | — | |
| UX-2.9 | Tab badge + Scan again | [ ] | UX-1.5 | |
| UX-2.10 | Pair comparison view | [ ] | UX-2.1 | |
| UX-3.1 | Settings sections | [ ] | — | |
| UX-3.2 | Presets | [ ] | UX-3.1 | |
| UX-3.3 | Field tooltips | [ ] | UX-3.1 | |
| UX-3.4 | Stale results hint | [ ] | — | |
| UX-3.5 | Unsaved settings guard | [ ] | — | |
| UX-3.6 | ORB settings (blocked on core) | [ ] | core ORB | |
| UX-4.1 | Reason legend | [ ] | — | |
| UX-4.2 | Softer metric copy | [ ] | — | |
| UX-5.1 | Keyboard shortcuts | [ ] | — | |
| UX-5.2 | Double-click open image | [ ] | UX-2.5 | |
| UX-5.3 | Modal a11y | [ ] | — | |
| UX-5.4 | Window title subtitle | [ ] | UX-1.1, UX-2.9 | |
| UX-6.1 | Export CSV | [ ] | — | |

---

## Notes for future agents

- **Do not** expand scope into backend pairwise progress unless UX-1.2 indeterminate pairwise bar is insufficient; file a separate core task referencing `ScanEvent` in `similar-textures-core/src/lib.rs`.
- Reuse existing validation in `validateSettings()` when adding presets (UX-3.2).
- Keep `hide_single_image_groups` behavior consistent between Settings and Results toggle (UX-2.4 vs UX-3.1).
- When splitting `+page.svelte`, pass props/events explicitly; avoid new global stores unless necessary.
- Document manual test steps in each PR using `test-images/` when available.
