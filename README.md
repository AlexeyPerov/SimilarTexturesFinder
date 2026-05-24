# Similar Textures Finder

Desktop app for finding duplicate and near-duplicate texture images in a folder. Point it at a library, run a scan, and browse grouped results with thumbnails, scores, and match reasons so you can review or clean up redundant assets.

Each image is checked in two stages:

- **File hash** — compares raw file bytes (SHA-256 by default) to catch exact duplicates, even when filenames differ.
- **Visual similarity** — for files that are not byte-identical, a composite score combines three metrics (weights are configurable in Settings):
  - **pHash (perceptual hash)** — a compact fingerprint of overall appearance; good at spotting near-duplicates and re-exports quickly.
  - **SSIM (structural similarity)** — compares luminance and structure after resize; catches images that look alike but may differ in compression or minor edits.
  - **Histogram** — compares color distribution across channels; helps group textures with similar palettes even when layout differs.

Optional preprocessing (alpha crop, rotation, and flip checks) can be enabled for textures with transparency or orientation variants.

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
