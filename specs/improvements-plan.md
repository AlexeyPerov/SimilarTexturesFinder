# Stability & Optimization Improvements Plan

## Overview

This document describes a phased plan to improve the Similar Textures tool in two dimensions:

- **Stability** — prevent crashes, memory exhaustion, and data races under large workloads.
- **Performance** — reduce wall-clock time and peak memory for typical scans (≥ 1000 images).

Each phase is designed to be independently shippable. Phase 1 alone delivers the majority of the benefit.

---

## Phase 1 — Feature-vector pre-computation (highest impact)

The single most impactful structural change: compute all per-image features once during ingestion, store only compact feature vectors, then drop the raw pixel buffers before pairwise begins.

### 1.1 New `Features` struct

**File:** `backend/src/features/mod.rs`

Replace the bare module with a struct that holds pre-computed, compact representations:

```rust
pub struct Features {
    pub phash: u64,                              // 8 bytes
    pub ssim_luma: Vec<f32>,                     // resize_size² × 4 bytes  (e.g. 256 KB at 256×256)
    pub histogram: Vec<f32>,                     // bins_per_ch³ × 4 bytes  (e.g. 512 × 4 = 2 KB)
    pub rotation_phashes: Vec<u64>,              // 0–8 × 8 bytes
    pub rotation_ssim_lumas: Vec<Vec<f32>>,      // 0–8 × resize_size² × 4 bytes
    pub rotation_histograms: Vec<Vec<f32>>,      // 0–8 × bins³ × 4 bytes
}
```

### 1.2 Compute during ingestion, not pairwise

**File:** `backend/src/main.rs` (ingestion loop, lines 110–154)

Currently the loop decodes each image and stores the full RGBA buffer in `Vertex::prepared`. Change to:

1. Decode image → `ImageData`.
2. Call `Features::extract(img, &cfg)` which:
   - Computes the pHash (64-bit integer).
   - Resizes to `resize_size × resize_size` and extracts the luma plane (for SSIM).
   - Builds the RGB histogram.
   - If rotations/flips are enabled, computes the same features for all transforms of the image.
3. Drop the `ImageData`; store only `Features` in the vertex.

**Vertex change** (`backend/src/vertex.rs`):

```rust
pub struct Vertex {
    pub path: PathBuf,
    pub digest: Vec<u8>,
    pub features: Option<Features>,   // was: prepared: Option<ImageData>
}
```

### 1.3 Rewrite pairwise to use features

**File:** `backend/src/pairwise.rs`

- **pHash pair:** Hamming distance of two `u64` values — ~1 ns. No DCT, no resize.
- **SSIM pair:** `global_ssim(&features_a.ssim_luma, &features_b.ssim_luma)` — no resize per pair.
- **Histogram pair:** compare two pre-built `Vec<f32>` — no resize per pair.
- **Rotations:** iterate `rotation_phashes` / `rotation_ssim_lumas` / `rotation_histograms` instead of creating new `ImageData` transforms.

This eliminates:
- ~50M redundant DCT computations at 10K images (pHash caching).
- ~100M redundant image resizes (SSIM + histogram, both images per pair).
- Up to 400M `ImageData` allocations when rotations + flips are enabled.

### 1.4 Memory reduction

Before phase 1 (per image at 256×256 RGBA): 256 KB pixel buffer + 256 KB clone in pairwise = **512 KB**.

After phase 1 (per image): ~8 bytes (pHash) + ~256 KB (SSIM luma) + ~2 KB (histogram) = **~258 KB**, and no clone for rayon.

Peak memory at 10K images: ~5.1 GB → ~2.5 GB (plus the pair list is eliminated in phase 2).

### 1.5 Files changed

| File | Change |
|------|--------|
| `features/mod.rs` | Add `Features` struct and `extract()` method |
| `features/phash.rs` (new) | Move pHash computation here, return `u64` |
| `features/ssim_luma.rs` (new) | Resize + luma extraction, return `Vec<f32>` |
| `features/histogram_feat.rs` (new) | Build histogram, return `Vec<f32>` |
| `vertex.rs` | Replace `prepared: Option<ImageData>` with `features: Option<Features>` |
| `main.rs` | Call `Features::extract()` in ingestion loop; drop `ImageData` |
| `pairwise.rs` | Operate on `Features` instead of `ImageData` |
| `similarity/phash.rs` | `compute()` takes `u64` hash pair, not images |
| `similarity/ssim.rs` | `compute()` takes `&[f32]` luma pair, not images |
| `similarity/histogram.rs` | `compute()` takes `&[f32]` histogram pair, not images |
| `similarity/rotations.rs` | Iterate feature vectors instead of pixel transforms |
| `similarity/prepare.rs` | Remove per-pair resize helpers (now done once) |

---

## Phase 2 — Eliminate O(n²) pair allocation

### 2.1 Lazy pair iteration

**File:** `backend/src/pairwise.rs` (lines 33–35)

Currently:

```rust
let pairs: Vec<(usize, usize)> = (0..n)
    .flat_map(|i| ((i + 1)..n).map(move |j| (i, j)))
    .collect();
```

At 10K images, this allocates ~800 MB before any work starts.

Replace with:

```rust
let results: Vec<_> = (0..n).into_par_iter().flat_map(|i| {
    let n = n; // capture
    (i + 1..n).map(move |j| {
        // compute pair (i, j) ...
    })
}).collect();
```

No intermediate `Vec` of pairs. Rayon's work-stealing scheduler handles distribution.

### 2.2 Prune score_cache

**File:** `backend/src/pairwise.rs` (lines 70–84)

Only insert into `score_cache` pairs that are:
- Hash-equal, OR
- Have `final_score > threshold` (potential composite edge).

Singleton pairs below threshold are never used again after the pairwise pass. This reduces `score_cache` from O(n²) to O(E) where E is the number of edges in the similarity graph (typically much smaller).

Apply the same pruning after clustering: keep only intra-group scores, drop the rest.

### 2.3 Remove duplicate data collections

**File:** `backend/src/main.rs` (lines 165–166)

```rust
let path_refs: Vec<PathBuf> = vertices.iter().map(|v| v.path.clone()).collect();
let digests: Vec<Vec<u8>> = vertices.iter().map(|v| v.digest.clone()).collect();
```

Pass `&vertices` directly to `build_groups()` and `group_scores()` instead of cloning path and digest data into separate Vecs.

### 2.4 Files changed

| File | Change |
|------|--------|
| `pairwise.rs` | Replace `pairs` Vec with lazy `into_par_iter().flat_map()` |
| `pairwise.rs` | Prune `score_cache` to only relevant entries |
| `main.rs` | Remove `path_refs` / `digests` clones; pass `&vertices` |
| `clustering.rs` | Accept `&[Vertex]` instead of `&[PathBuf]` |
| `group_score.rs` | Accept `&[Vertex]` instead of separate `&[Vec<u8>]` |

---

## Phase 3 — Stability hardening

### 3.1 Graceful cancellation (SIGINT / SIGTERM)

**New dependency:** `ctrlc` crate.

- Register a `ctrlc::set_handler` at startup that sets an `AtomicBool`.
- Check the flag between work items in the ingestion loop and between rayon batches.
- On cancellation: remove the partial temp output file, exit with non-zero code + stderr message.

### 3.2 File-hash early grouping

**File:** `backend/src/main.rs`

After ingestion, group vertices by digest **before** pairwise. For each group of k identical files, emit hash edges for all C(k,2) pairs immediately and skip those pairs in the expensive pairwise pass.

This avoids running SSIM/pHash/histogram on identical files, which is a common case in game asset collections with duplicated textures.

### 3.3 Canonicalize input paths during scan

**File:** `backend/src/scanner.rs`

Resolve symlinks and canonicalize paths during scanning (not just at output). This prevents the same physical file from appearing as multiple vertices with different symlink paths.

### 3.4 Atomic write robustness on Windows

**File:** `backend/src/output.rs`

The `tempfile::NamedTempFile::persist()` call can fail on Windows if the target file is already open. Add a retry loop with a short sleep, and fall back to non-atomic overwrite if rename fails after N attempts.

### 3.5 Files changed

| File | Change |
|------|--------|
| `Cargo.toml` | Add `ctrlc` dependency |
| `main.rs` | Add cancellation flag, check in loops, cleanup on cancel |
| `scanner.rs` | Canonicalize paths during scan |
| `pairwise.rs` | Skip hash-equal groups in pairwise loop |
| `output.rs` | Retry + fallback for Windows rename |

---

## Phase 4 — Flutter frontend improvements

### 4.1 Thumbnail resolution limiting (P2)

**File:** `frontend/lib/screens/results_screen.dart:197`

```dart
Image.file(file, fit: BoxFit.cover, cacheWidth: 256, cacheHeight: 256)
```

Without `cacheWidth`/`cacheHeight`, Flutter decodes the full-resolution image for each grid tile. A 4K texture decoded 16 times in a 4×4 grid = ~1 GB of GPU memory. Adding `cacheWidth: 256` makes each decode cost ~256 KB instead.

### 4.2 Cancel button during scan (P2)

**File:** `frontend/lib/screens/scan_screen.dart:326-329`

Replace the bare `LinearProgressIndicator` with a row containing the indicator and a "Cancel" button:

```dart
if (_running) ...[
  const SizedBox(height: 8),
  Row(children: [
    const Expanded(child: LinearProgressIndicator()),
    const SizedBox(width: 12),
    OutlinedButton(
      onPressed: () => killActiveTextureTool(),
      child: const Text('Cancel'),
    ),
  ]),
],
```

### 4.3 Async JSON parsing (P2)

**File:** `frontend/lib/services/json_parser.dart:23-26`

```dart
static Future<ScanResult> parseFileAsync(String path) async {
    final text = await File(path).readAsString();
    return parseString(text);
}
```

Call from `_runScan()` with `await ScanResult.parseFileAsync(outFile)` instead of the synchronous `parseFile()`. For very large result files (10K+ groups), this prevents UI jank.

### 4.4 Scoped rebuilds for results (P3)

**File:** `frontend/lib/screens/results_screen.dart:36-38`

Replace `ListenableBuilder(listenable: widget.appState)` with a more targeted approach. Options:

- `ValueListenableBuilder` wrapping only `lastScanResult`.
- A `Selector<AppState, ScanResult?>` that only rebuilds when the result changes.
- A dedicated `ValueNotifier<ScanResult?>` on `AppState`.

This prevents the entire GridView from rebuilding when unrelated state (theme, settings) changes.

### 4.5 Process lifecycle safety (P3)

**File:** `frontend/lib/services/backend_runner.dart:6`

Wrap the global `_activeTextureToolProcess` in a dedicated `ScanSession` class:

```dart
class ScanSession {
    Process? _process;
    bool get isRunning => _process != null;
    void attach(Process p) { _process = p; }
    void kill() { _process?.kill(ProcessSignal.sigkill); _process = null; }
}
```

Inject it into `ScanScreen` so the lifecycle is explicit and testable, instead of a global mutable.

### 4.6 Files changed

| File | Change |
|------|--------|
| `results_screen.dart` | Add `cacheWidth`/`cacheHeight` to `Image.file` |
| `results_screen.dart` | Scope rebuilds with `Selector` or `ValueListenableBuilder` |
| `scan_screen.dart` | Add cancel button row |
| `json_parser.dart` | Add `parseFileAsync()` method |
| `backend_runner.dart` | Encapsulate process reference in `ScanSession` class |

---

## Phase 5 — Minor optimizations (nice-to-haves)

### 5.1 Cache lossy path strings for sorting

**File:** `backend/src/scanner.rs:42`

```rust
out.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));
```

Each comparison allocates two Strings. Pre-compute:

```rust
let mut tagged: Vec<(String, PathBuf)> = out.into_iter()
    .map(|p| (p.to_string_lossy().into_owned(), p))
    .collect();
tagged.sort_by(|a, b| a.0.cmp(&b.0));
let out: Vec<PathBuf> = tagged.into_iter().map(|(_, p)| p).collect();
```

Apply the same pattern in `clustering.rs:64`.

### 5.2 Pre-group digests for group scoring

**File:** `backend/src/group_score.rs:36`

Build `HashMap<Vec<u8>, Vec<usize>>` once, then check digest equality via HashMap lookup instead of linear scan + byte comparison.

### 5.3 SSIM floating-point precision

**File:** `backend/src/similarity/ssim.rs:55-82`

SSIM is computed in `f32`. For images with very similar means/variances, `f32` precision loss can produce slightly negative SSIM values that are then clamped to 0. Consider computing in `f64` (negligible cost on modern CPUs) for improved numerical stability, especially for dark/low-contrast textures.

### 5.4 Files changed

| File | Change |
|------|--------|
| `scanner.rs` | Pre-compute lossy strings before sort |
| `clustering.rs` | Same pattern |
| `group_score.rs` | Build digest HashMap |
| `similarity/ssim.rs` | Switch to `f64` accumulation |

---

## Dependency & risk summary

| Phase | New dependencies | Risk | Estimated effort |
|-------|-----------------|------|------------------|
| 1 | None | Medium — largest refactor, touches core pipeline | 2–3 days |
| 2 | None | Low — mostly removing allocations | 0.5–1 day |
| 3 | `ctrlc` | Low — additive changes | 1 day |
| 4 | None | Low — Flutter-level changes | 1 day |
| 5 | None | Very low | 0.5 day |

Phases 2–5 can be done in any order after Phase 1. Phase 1 is the prerequisite because it changes the data model that later phases rely on.

---

## Acceptance criteria

After all phases:

- [ ] 10K images scanned in under 5 minutes on 8-core machine (vs. current ~30+ minutes estimated).
- [ ] Peak memory < 4 GB for 10K images at default settings (vs. current unbounded).
- [ ] Rotations + flips enabled on 1K images completes in under 2 minutes (currently impractical).
- [ ] Ctrl+C during scan exits cleanly within 2 seconds, no orphan temp files.
- [ ] Flutter results grid loads without jank for 500+ groups.
- [ ] Cancel button stops scan and returns UI to idle state.
