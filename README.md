# Similar Textures

MVP backend: Rust CLI `texture_tool` scans a folder, scores image pairs (pHash, SSIM, histogram), clusters similar textures, and writes `result.json`.

## Backend build

```bash
cd backend
cargo build --release
```

Example:

```bash
./target/release/texture_tool \
  --input /path/to/folder \
  --output /tmp/result.json \
  --config backend/config.example.json \
  --threads 8
```

- Exit `0` only after a successful atomic write of `result.json`. Errors go to stderr ([Task.md](Tasks/Task.md) §3.9).
- `groups[].images` paths are **absolute** (canonicalized).

## Clustering / singletons

Every scanned vertex (image with readable bytes for hashing) appears in **exactly one** group. **Singleton groups** (size 1) are included; their `score` field is omitted per §6.

## Metrics

- **SSIM**: global single-window SSIM on grayscale luma after resize. `ssim_threshold` **gates** the metric: if SSIM `<` threshold, that metric is **invalid** for §5 renormalization (not the same as the global `threshold` on `final_score`).
- **Histogram**: `hist_bins` maps to per-channel bins via cube root (e.g. 512 → 8³). Methods: `correlation` (Pearson mapped to [0,1]) or `bhattacharyya`.
- **Rotations**: when `enable_rotations`, B is tried at 0°/90°/180°/270°. When `enable_flip`, each rotation is paired with a horizontal-flip variant (max over all). pHash, SSIM, and histogram all use this wrapper.

See [Tasks/Task.md](Tasks/Task.md) and [Tasks/Task_similarity_appendix.md](Tasks/Task_similarity_appendix.md) for normative detail.
