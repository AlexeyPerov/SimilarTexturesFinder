# Tauri Windows Runtime Checklist (Deferred Execution)

Use this checklist on a Windows machine to complete Phase 6 Windows runtime validation.

## Preconditions

- Windows 10/11 with WebView2 runtime installed.
- Rust toolchain installed (`rustup`, `cargo`).
- Node.js + npm installed.
- Repository checked out with current migration branch.

## Setup

```powershell
cd app
npm install
```

```powershell
cd app/src-tauri
cargo test
```

```powershell
cd app
npm run check
```

## Runtime smoke test

```powershell
cd app
npm run tauri dev
```

Perform the following in the running app:

1. Open app and confirm it starts without crash.
2. Confirm only two tabs exist: `Scan`, `Results`.
3. Open settings modal from gear button.
4. Edit a setting, click `Cancel`; reopen settings and confirm value was not applied.
5. Edit setting again, click `Save`; restart app and confirm persisted value loaded.
6. In `Scan`, choose an input folder.
7. Click `Run Scan`; confirm console logs stream only in `Scan`.
8. While running, click `Cancel`; confirm status transitions out of running and no crash.
9. Run scan to completion; confirm active tab auto-switches to `Results`.
10. Click `Export JSON`; save file and verify JSON exists and has `groups` payload.

## Pass/fail template

| Check | Result (PASS/FAIL) | Notes |
|---|---|---|
| App boot |  |  |
| Settings cancel path |  |  |
| Settings save persistence across restart |  |  |
| Scan start/log stream |  |  |
| Cancel path |  |  |
| Auto-switch to Results on success |  |  |
| Export JSON output correctness |  |  |
| Port 1440 strict binding behavior |  |  |

## Path and dialog notes (Windows-specific)

- Validate backslash paths in scan input and exported output path.
- Verify file picker and save dialog return usable absolute paths.
- Verify image thumbnail loading works with Windows-style paths (`C:\\...`).

## Troubleshooting

- If `npm run tauri dev` fails on missing WebView2, install Microsoft Edge WebView2 Runtime and retry.
- If port `1440` is occupied, free the port and retry (strict port is intentionally enabled).
- If a scan fails immediately, verify selected folder exists and contains supported raster formats.

