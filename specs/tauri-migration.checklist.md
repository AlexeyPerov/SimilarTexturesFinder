# Tauri Migration Execution Checklist

Use this file as the implementation tracker for `specs/tauri-migration.plan.md`.

## How To Use

- One PR per checklist item unless noted.
- Keep PRs small and reviewable.
- Do not start an item until dependencies are complete.
- Mark each item:
  - `[ ]` not started
  - `[~]` in progress
  - `[x]` done
- Add PR link, owner, and date in the item notes.

---

## Milestone 0 - Foundation

### M0.1 Scaffold `app/` (Tauri 2 + Svelte 5)
- Status: [ ]
- Depends on: none
- Owner:
- PR:
- Notes:
  - Create `app/src`, `app/src-tauri`, base scripts.
  - Confirm `npm run tauri dev` boots.
- Verification:
  - [ ] Dev app window opens.
  - [ ] Build/check scripts run without scaffold errors.

### M0.2 Enforce dev port 1440
- Status: [ ]
- Depends on: M0.1
- Owner:
- PR:
- Notes:
  - Set Vite `server.port = 1440`.
  - Set `server.strictPort = true`.
  - Align Tauri `devUrl` to `http://localhost:1440`.
- Verification:
  - [ ] Dev server binds to `1440`.
  - [ ] Occupied `1440` causes fail-fast (no fallback).

---

## Milestone 1 - Rust Core Extraction

### M1.1 Create `similar-textures-core` crate
- Status: [ ]
- Depends on: M0.2
- Owner:
- PR:
- Notes:
  - Add crate under `app/src-tauri/crates/similar-textures-core`.
  - Port backend domain modules into crate.
  - Remove CLI parsing concerns from core.
- Verification:
  - [ ] Core crate compiles independently.
  - [ ] Public API exposes scan request/result types.

### M1.2 Preserve behavior parity via regression tests
- Status: [ ]
- Depends on: M1.1
- Owner:
- PR:
- Notes:
  - Add fixture-driven tests for grouping/scoring/path semantics.
  - Validate output assumptions used by current product.
- Verification:
  - [ ] Regression suite passes on CI/local.
  - [ ] No known parity regressions documented as unresolved.

---

## Milestone 2 - Tauri Host API

### M2.1 Implement command surface
- Status: [ ]
- Depends on: M1.2
- Owner:
- PR:
- Notes:
  - Add commands for start/cancel/status/get-result/export/settings load-save.
  - Add centralized app state in `src-tauri`.
  - Prevent concurrent scan starts.
- Verification:
  - [ ] Commands callable from frontend.
  - [ ] Typed success/error responses are stable.

### M2.2 Implement events + cancellation wiring
- Status: [ ]
- Depends on: M2.1
- Owner:
- PR:
- Notes:
  - Emit `scan-log`, `scan-progress`, `scan-finished`.
  - Wire cancellation token into core scan loop.
- Verification:
  - [ ] Logs/progress stream during scan.
  - [ ] Cancel transitions scan to non-running safely.

---

## Milestone 3 - Svelte UI

### M3.1 Build shell (tabs + settings modal)
- Status: [ ]
- Depends on: M2.2
- Owner:
- PR:
- Notes:
  - Two tabs only: `Scan`, `Results`.
  - Settings is modal popup.
  - Apply reference design language.
- Verification:
  - [ ] No settings tab exists.
  - [ ] Shell visually matches target style direction.

### M3.2 Implement Scan tab
- Status: [ ]
- Depends on: M3.1
- Owner:
- PR:
- Notes:
  - Folder picker, run/cancel controls.
  - Console with clear/copy.
  - Explicit-run only (no auto-rescan).
- Verification:
  - [ ] Scan starts only from explicit user action.
  - [ ] Logs are shown only in `Scan` tab.

### M3.3 Implement Results tab + auto-activate on success
- Status: [ ]
- Depends on: M3.2
- Owner:
- PR:
- Notes:
  - Render result groups/thumbnails.
  - Empty state for no result.
  - Auto-switch to `Results` on successful scan.
- Verification:
  - [ ] Successful scan always activates `Results`.
  - [ ] Results remain available in memory until next run/app restart.

### M3.4 Add `Export JSON`
- Status: [ ]
- Depends on: M3.3
- Owner:
- PR:
- Notes:
  - Add export action in `Results`.
  - Save using native dialog + host command.
  - Export from in-memory result.
- Verification:
  - [ ] Exported file is valid JSON.
  - [ ] Export content matches current in-memory result.

---

## Milestone 4 - Settings Persistence

### M4.1 Define and implement settings schema
- Status: [ ]
- Depends on: M2.1
- Owner:
- PR:
- Notes:
  - Include analysis settings parity with legacy config model.
  - Add schema version for future changes.
- Verification:
  - [ ] Load/save round-trip works.
  - [ ] No legacy migration logic exists.

### M4.2 Wire modal settings workflow
- Status: [ ]
- Depends on: M4.1, M3.1
- Owner:
- PR:
- Notes:
  - Save/cancel flows.
  - Validation errors displayed cleanly.
  - No implicit scan trigger on save.
- Verification:
  - [ ] Settings persist across restarts.
  - [ ] Saving settings never auto-starts scan.

---

## Milestone 5 - Legacy Removal

### M5.1 Remove legacy Flutter and old backend CLI layout
- Status: [ ]
- Depends on: M3.4, M4.2
- Owner:
- PR:
- Notes:
  - Remove old `frontend/`.
  - Remove old backend entrypoint/layout after core is fully in app.
  - Keep only necessary shared assets/docs.
- Verification:
  - [ ] Repo has one active app in `app/`.
  - [ ] No docs/scripts reference legacy runtime path.

### M5.2 Refresh documentation/runbook
- Status: [ ]
- Depends on: M5.1
- Owner:
- PR:
- Notes:
  - Update root README and contributor instructions.
  - Document port `1440` and conflict handling.
- Verification:
  - [ ] Fresh clone setup works from docs.
  - [ ] Team runbook reflects new architecture.

---

## Milestone 6 - Platform Validation

### M6.1 Runtime validation for macOS + Windows
- Status: [ ]
- Depends on: M5.2
- Owner:
- PR:
- Notes:
  - Smoke test scan/cancel/results/export/settings on both OSes.
  - Validate dialogs/path behavior per platform.
- Verification:
  - [ ] macOS runtime checklist passed.
  - [ ] Windows runtime checklist passed.
  - [ ] Known issues documented for later installer/signing pass.

---

## Final Go/No-Go Checklist

- [ ] Single app architecture in `app/`.
- [ ] Tauri 2 + Svelte 5 confirmed.
- [ ] Dev port fixed to `1440` (`strictPort` enabled).
- [ ] Two tabs (`Scan`, `Results`) and settings modal implemented.
- [ ] Logs only in `Scan`.
- [ ] No auto-rescan behavior anywhere.
- [ ] Auto-switch to `Results` after successful scan.
- [ ] In-memory results + JSON export works.
- [ ] Legacy frontend/backend runtime removed.
- [ ] macOS + Windows runtime validation complete.
