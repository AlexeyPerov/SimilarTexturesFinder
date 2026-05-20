# Similar Textures

Single desktop application built with **Tauri 2 + Svelte 5 + Rust core**.

The previous split architecture (Flutter frontend + standalone Rust CLI backend) has been removed. The runtime entrypoint is now the `app/` workspace only.

## Repository layout

- `app/` — active application workspace
  - `src/` — Svelte UI (`Scan`, `Results`, settings modal)
  - `src-tauri/` — Tauri host commands/events
  - `src-tauri/crates/similar-textures-core/` — Rust scan/similarity core
- `specs/` — migration plans, checklists, and validation artifacts
- `test-images/` — optional test fixture images for scan checks

## Requirements

- Node.js + npm
- Rust toolchain (`rustup`, `cargo`)
- Tauri prerequisites for your OS (WebView/runtime and toolchain requirements)

## Development (app-only workflow)

```bash
cd app
npm install
npm run tauri dev
```

## Checks

Frontend checks:

```bash
cd app
npm run check
```

Rust host/core checks:

```bash
cd app/src-tauri
cargo test
```

## Port configuration

The Svelte/Vite dev server is pinned to **port `1440`** with strict binding:

- `server.port = 1440`
- `server.strictPort = true`

If port `1440` is occupied, startup fails instead of falling back to another port.

## Runtime behavior

- `Scan` and `Results` are the only top-level tabs.
- `Settings` is a modal popup.
- Scan is explicit (no auto-rescan on settings edits).
- Scan logs are displayed only on `Scan`.
- On successful scan, UI auto-switches to `Results`.
- Scan results are stored in memory; `Export JSON` writes the current in-memory result to disk.

## Settings persistence

Settings are persisted by the Tauri host under the app config directory as `settings.json`.

- Current storage is schema-versioned.
- Legacy unversioned settings payloads are still accepted on read.
- No migration from old Flutter user data is supported or required.

## Platform status

- Runtime target: macOS + Windows
- macOS runtime validation is included in the migration artifacts under `specs/`.
- Windows packaging/signing polish remains deferred to a later pass.
