# Similar Textures Finder

Single desktop application built with **Tauri 2 + Svelte 5 + Rust core**.

## Repository layout

- `app/` — application workspace (Svelte UI, Tauri host, Rust core)
- `specs/` — plans, checklists, and validation artifacts
- `test-images/` — optional test fixture images

## Requirements

- Node.js + npm
- Rust toolchain (`rustup`, `cargo`)
- Tauri prerequisites for your OS (WebView/runtime and toolchain requirements)

## Run

```bash
cd app
npm install
npm run tauri dev
```

The Vite dev server is pinned to port `1440`. If startup fails, check that nothing else is using that port.

## Build

```bash
cd app
npm install
npm run tauri build
```

Installers and bundles are written under `app/src-tauri/target/release/bundle/`.

## Checks

```bash
cd app
npm run check
cd src-tauri && cargo test
```
