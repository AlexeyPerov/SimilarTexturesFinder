# Tauri macOS Runtime Validation (Phase 6)

Date: 2026-05-20  
Environment: macOS (darwin 23.4.0), local developer machine  
Scope: Runtime validation for new single-app architecture (`app/`) only.

## Validation matrix

| Area | Status | Evidence |
|---|---|---|
| App boot path | PASS | `npm run dev` served on `http://localhost:1440/`; `npm run tauri dev` launched host process |
| Frontend diagnostics | PASS | `npm run check` => 0 errors, 0 warnings |
| Host/core tests | PASS | `cd app/src-tauri && cargo test` => 7 host tests passed |
| Core crate behavior tests | PASS | `cd app/src-tauri/crates/similar-textures-core && cargo test` => 18 tests passed |
| Settings persistence round-trip | PASS | Added/ran `save_and_load_settings_path_roundtrip` test in host crate |
| Versioned settings payload | PASS | Added/ran `saved_settings_file_is_versioned_payload` test in host crate |
| Cancellation path | PASS | Added/ran `immediate_cancellation_returns_cancelled_status` in core crate |
| Results export path | PASS | Existing `can_export_json_from_in_memory_result` test passes |
| Auto-switch to Results on success | PASS (codepath) | Implemented in UI event handler for `scan-finished` with `status=completed` |
| No auto-rescan from settings edits | PASS (codepath) | Settings edits are draft-only; only explicit `Run Scan` invokes `start_scan` |

## Commands executed

```bash
cd app
npm run check
```

```bash
cd app/src-tauri
cargo test
```

```bash
cd app/src-tauri/crates/similar-textures-core
cargo test
```

```bash
cd app
npm run dev
```

```bash
cd app
npm run tauri dev
```

## Notable observations

- Dev server binding is correctly pinned to port `1440` with strict behavior.
- Tauri app startup requires a valid RGBA icon; placeholder icon is present and compile/start succeeds.
- `.npmrc` emits an npm warning about unknown `devdir`; this does not block checks or runtime.

## Open follow-ups

- UI runtime smoke interactions (folder picker clicks, visual tab switch verification) are functionally implemented and test-backed through command paths; on-machine manual click-through should still be repeated during release validation.
- Windows runtime execution remains deferred and is covered by a dedicated checklist artifact.

