# Tauri + Svelte Migration Plan

## Overview

This document defines the migration from the current Flutter desktop frontend + Rust CLI backend split to a single desktop application built with:

- **Tauri 2** (desktop shell + native bridge)
- **Svelte 5** (UI)
- **Rust core** (scan/similarity engine, called in-process)

The migration is intentionally aggressive: legacy Flutter frontend and legacy backend CLI layout are removed after parity is reached.

---

## Product Requirements

## Functional Requirements

- The app is a **single desktop app** with no separate backend service process.
- UI uses the same design language as the reference app (`vibe-launcher`): dark palette, compact controls, tabbed layout, console styling.
- Main app navigation has exactly two tabs:
  - `Scan`
  - `Results`
- `Settings` is a modal popup (not a dedicated tab/page).
- Scan behavior:
  - User selects input folder.
  - User starts scan explicitly.
  - No auto-rescan on any setting/path change.
  - Scan can be cancelled.
  - Console logs are visible only in `Scan`.
- Result behavior:
  - On successful scan, `Results` tab becomes active automatically.
  - Results are kept in memory as the source of truth.
  - User can export current in-memory result to JSON via `Export JSON`.
- Configuration behavior:
  - Fresh Tauri-based settings storage only.
  - No migration of old Flutter user data.

## Non-Functional Requirements

- Target runtime platforms for this migration: **macOS + Windows**.
- Windows packaging/code-signing optimization can be deferred.
- Keep behavior stable for current algorithm output semantics.
- Avoid legacy leftovers after cutover (no dual frontend support).

## Tooling and Dev Requirements

- Tauri version: **2.x**
- Default local dev port: **1440**
  - Rationale: 1420 and 1430 are already occupied.
  - Configure Vite with:
    - `server.port = 1440`
    - `server.strictPort = true`
  - Ensure Tauri dev URL points to port `1440`.

---

## Target Architecture

## Repository Layout (Target State)

```text
/
  app/
    package.json
    vite.config.ts (or .js)
    src/                      # Svelte UI
      routes/+page.svelte     # top-level shell (or equivalent component split)
      lib/
        components/
        stores/
        models/
        services/
    src-tauri/
      Cargo.toml
      tauri.conf.json
      src/
        main.rs
        commands/
        state.rs
      crates/
        similar-textures-core/
          Cargo.toml
          src/
            lib.rs
            ...ported backend modules...
  specs/
  test-images/
```

## Runtime Architecture

### 1) UI Layer (Svelte)

- Owns visual state, tab switching, modal visibility, user interactions.
- Subscribes to scan progress/log events from Tauri.
- Maintains only UI-facing state; authoritative scan output comes from backend state.

### 2) Tauri Host Layer (`src-tauri`)

- Exposes typed commands for scan lifecycle, settings, and export.
- Emits events for progress/log streaming.
- Owns process-safe app state:
  - current scan status
  - cancellation token/flag
  - last scan result in memory

### 3) Core Engine Layer (`similar-textures-core`)

- Pure Rust domain layer migrated from current backend modules.
- No CLI argument parsing.
- API-first design, callable from Tauri host directly.
- Deterministic result model matching existing behavior expectations.

## Data Flow

1. User clicks `Run Scan` in `Scan` tab.
2. UI invokes `start_scan(request)`.
3. Host validates request, starts core scan on worker context.
4. Host emits progress/log events during execution.
5. On success:
   - host stores result in memory,
   - host emits completion event,
   - UI switches active tab to `Results`.
6. On `Export JSON`, UI invokes `export_result_json(path)` or dialog-backed command; host serializes in-memory result.

---

## API Contract (Planned)

## Tauri Commands

- `start_scan(request)` -> starts a scan if not already running.
- `cancel_scan()` -> requests cancellation of active scan.
- `scan_status()` -> returns current status (`idle|running|completed|failed|cancelled`).
- `get_last_result()` -> returns in-memory result if present.
- `export_last_result_json(target_path?)` -> writes result JSON to user-selected path.
- `load_settings()` -> returns persisted app settings.
- `save_settings(settings)` -> persists app settings.

## Tauri Events

- `scan-log` (line/chunk, stream type)
- `scan-progress` (phase + counters/percent where available)
- `scan-finished` (success/failure/cancelled metadata)

Event payloads must be typed and versionable to avoid frontend/backend drift.

---

## UI/UX Requirements

## Tab Shell

- Two top-level tabs only: `Scan`, `Results`.
- `Results` is initially disabled or empty-state until first successful run.
- Auto-switch to `Results` after successful scan completion.

## Settings Modal

- Gear icon (or equivalent) opens modal.
- Modal contains all tunables currently represented by `AnalysisConfig`.
- Save is explicit; changes do not trigger scan automatically.

## Scan Tab

- Input folder picker.
- `Run Scan` and `Cancel` controls.
- Console viewer with clear/copy options.
- Logs shown only here.

## Results Tab

- Group browsing UI with thumbnails and group metadata.
- Empty state when no result exists.
- `Export JSON` button exports current in-memory result.

---

## Migration Phases and Agent Task Cards

Each task card is intended to be implemented by an agent in a focused PR.

## Phase 0 - Foundation and Guardrails

### Task 0.1 - Create `app/` workspace (Tauri 2 + Svelte 5)

**Goal**
- Scaffold new app in `app/` subfolder with Tauri 2 and Svelte 5.

**Inputs**
- This plan.
- Reference visual language from `vibe-launcher`.

**Implementation**
- Initialize Tauri + Svelte project.
- Establish base scripts for dev/build/check.
- Ensure app runs in dev mode.

**Deliverables**
- `app/src`, `app/src-tauri`, `app/package.json`, Tauri configs.

**Done Criteria**
- `npm run tauri dev` starts successfully.
- Window opens with starter shell.

---

### Task 0.2 - Enforce dev port 1440

**Goal**
- Ensure the app always uses port `1440` in development.

**Implementation**
- Update Vite config to:
  - `port: 1440`
  - `strictPort: true`
- Align Tauri `devUrl` to `http://localhost:1440`.

**Deliverables**
- Updated `vite.config.*`.
- Updated `tauri.conf.json`.

**Done Criteria**
- Dev server binds to `1440`.
- If `1440` is busy, startup fails clearly (no fallback).

---

## Phase 1 - Rust Core Extraction

### Task 1.1 - Introduce `similar-textures-core` crate

**Goal**
- Move backend domain logic into reusable Rust crate under `app/src-tauri/crates/`.

**Implementation**
- Create crate and migrate algorithm modules (`scanner`, `pairwise`, `clustering`, `config`, etc.).
- Remove CLI-only concerns from core.
- Define clean `run_scan(request, callbacks, cancel)` API.

**Deliverables**
- New core crate with tests compiling.

**Done Criteria**
- Core crate builds independently.
- Legacy CLI parser is not required by core API.

---

### Task 1.2 - Preserve result schema semantics

**Goal**
- Ensure output behavior remains equivalent to existing expectations.

**Implementation**
- Keep group semantics, score behavior, and path normalization rules aligned with current backend contract.
- Add regression tests with fixture images where possible.

**Deliverables**
- Regression tests for key behavior.

**Done Criteria**
- Core scan on representative fixture set produces expected grouping behavior.

---

## Phase 2 - Tauri Host and App State

### Task 2.1 - Command surface for scan lifecycle

**Goal**
- Implement command handlers for start/cancel/status/result/export/settings.

**Implementation**
- Add command modules in `src-tauri/src/commands`.
- Create shared app state container in `src-tauri/src/state.rs`.
- Guard against concurrent scan starts.

**Deliverables**
- Working command API accessible from frontend.

**Done Criteria**
- Frontend can call commands and receive typed responses.

---

### Task 2.2 - Event streaming and cancellation

**Goal**
- Stream logs/progress to UI and support responsive cancellation.

**Implementation**
- Emit `scan-log`, `scan-progress`, `scan-finished`.
- Wire cancellation token into core run loop.

**Deliverables**
- Stable event bridge during long scans.

**Done Criteria**
- Cancel stops scan and transitions to non-running state without crash.

---

## Phase 3 - Svelte UI Shell and Behavior

### Task 3.1 - Build application shell with two tabs + settings modal

**Goal**
- Create production shell with required navigation model.

**Implementation**
- Tabs: `Scan`, `Results`.
- Modal settings panel, vibe-launcher-like design language.
- Shared dark theme tokens and reusable controls.

**Deliverables**
- Rendered shell with navigation and modal behavior.

**Done Criteria**
- No dedicated settings tab/page exists.

---

### Task 3.2 - Implement Scan tab UX

**Goal**
- Deliver scan orchestration UI with console.

**Implementation**
- Folder picker.
- Run/cancel controls.
- Console log panel with clear/copy.
- Explicit run only; no auto-rescan logic.

**Deliverables**
- Fully wired `Scan` tab.

**Done Criteria**
- Editing settings or paths never starts scan automatically.

---

### Task 3.3 - Implement Results tab UX + auto-switch

**Goal**
- Show in-memory results and switch tab automatically after success.

**Implementation**
- Render groups and thumbnails.
- Handle empty states.
- Listen for success completion event and set active tab to `Results`.

**Deliverables**
- Functional `Results` tab with post-scan auto-activation.

**Done Criteria**
- Successful scan always activates `Results`.

---

### Task 3.4 - Implement Export JSON

**Goal**
- Export current in-memory result to JSON.

**Implementation**
- Add `Export JSON` control in `Results`.
- Use native save dialog flow through Tauri host.
- Serialize from host-stored result object.

**Deliverables**
- Export path selection + file write.

**Done Criteria**
- Exported JSON exists and matches current in-memory result.

---

## Phase 4 - Settings Model and Persistence

### Task 4.1 - Define settings schema

**Goal**
- Create new canonical settings schema for Tauri app.

**Implementation**
- Include algorithm settings equivalent to existing `AnalysisConfig`.
- Add UI preferences required by new app only.
- Version schema for future changes.

**Deliverables**
- Typed settings models in frontend + backend.

**Done Criteria**
- Settings round-trip load/save without migration layer.

---

### Task 4.2 - Modal-driven settings workflow

**Goal**
- Finalize settings interaction model in modal.

**Implementation**
- Open/edit/save/cancel flows.
- Validation and user-friendly error handling.
- No implicit scan side effects.

**Done Criteria**
- Saving settings updates persistent state only.

---

## Phase 5 - Legacy Removal and Repo Cleanup

### Task 5.1 - Remove Flutter frontend and old backend CLI layout

**Goal**
- Complete aggressive cutover and remove obsolete code.

**Implementation**
- Delete legacy `frontend/`.
- Remove obsolete backend executable entrypoint/layout after core is ported.
- Keep only structures required by new app and shared assets/specs.

**Deliverables**
- Repository no longer contains active legacy app codepaths.

**Done Criteria**
- Build/docs/scripts reference only `app/`.

---

### Task 5.2 - Documentation and runbook refresh

**Goal**
- Make project operable from fresh clone.

**Implementation**
- Rewrite root README with new architecture and commands.
- Add troubleshooting for port `1440` conflicts and platform notes.

**Done Criteria**
- New contributor can run app using docs only.

---

## Phase 6 - Platform Validation (macOS + Windows Runtime)

### Task 6.1 - Runtime validation matrix

**Goal**
- Verify cross-platform runtime behavior for macOS and Windows.

**Implementation**
- Smoke test scan, cancel, results, export, settings persistence.
- Validate path handling and file dialogs per platform.

**Done Criteria**
- Both platforms pass runtime checklist.

**Deferred**
- Signing/installer polish and distribution hardening.

---

## Acceptance Criteria

- [ ] `app/` subfolder contains the only active application.
- [ ] Tauri 2 + Svelte 5 stack is used.
- [ ] Default dev port is `1440` with strict binding.
- [ ] Only `Scan` and `Results` tabs exist.
- [ ] Settings are provided via modal popup.
- [ ] Scan logs are visible only in `Scan`.
- [ ] No automatic scan start from settings/path changes.
- [ ] Successful scan auto-activates `Results`.
- [ ] Results are stored in memory and can be exported to JSON.
- [ ] Legacy Flutter/frontend split is removed.
- [ ] App runs on macOS and Windows (runtime validation complete).

---

## Risks and Mitigations

- **Risk:** Large refactor causes behavior drift in scan outputs.  
  **Mitigation:** Add regression fixtures/tests during core extraction.

- **Risk:** UI/backend contract mismatch (commands/events).  
  **Mitigation:** Define typed payloads and shared contract tests.

- **Risk:** Cancellation race conditions.  
  **Mitigation:** Centralized scan state machine and idempotent cancel handling.

- **Risk:** Port conflicts in dev environments.  
  **Mitigation:** enforce `1440` + `strictPort`, document resolution steps.

---

## Out of Scope for This Migration

- Linux target support.
- Installer/signing polish.
- Migration of legacy user preferences/data.
- New algorithmic features unrelated to parity/migration.
