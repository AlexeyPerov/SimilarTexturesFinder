# Execution plan — Similar Texture Finder

Use this checklist when implementing the project. Normative contracts live in [Task.md](Task.md); metric pipelines and trait detail in [Task_similarity_appendix.md](Task_similarity_appendix.md). Attachment hints in [AGENTS.md](AGENTS.md).

Complete tasks **in order** unless you explicitly parallelize independent scaffolding (e.g. Flutter layout can start after CLI + JSON contracts are frozen).

When each task is finished then append [DONE] to its title.

---

## Phase 1 — MVP (backend-first)

### 1. Repository and backend skeleton [DONE]

**Title:** Create `backend/` Rust binary crate with CLI wiring

**Description**

- What to do: Initialize a Cargo project under `backend/` with binary name aligned with docs (e.g. `texture_tool` per [Task.md](Task.md) §2). Parse `--input`, `--output`, `--config`, optional `--threads` per §3.4. Stub subcommands/logic paths: load config path, validate folder exists, exit non-zero with stderr message on fatal parse errors per §3.9.
- Result: `cargo run` prints usage or accepts args; missing required args exits non-zero; no real analysis yet.
- Files/folders: `backend/Cargo.toml`, `backend/src/main.rs`; later modules as empty `mod` files if useful.

**Context (read first)**

- [Task.md](Task.md) §2 (integration), §3.4 (CLI), §3.9 (exit/stderr), §10 (deliverables)
- [AGENTS.md](AGENTS.md)

---

### 2. Config contract — deserialize and defaults [DONE]

**Title:** Implement `config.json` model and defaults

**Description**

- What to do: Define structs (e.g. `Config` in `backend/src/config.rs`) matching §5 field list: `enable_*`, `threshold`, `weights`, optional `hash_algorithm`, metric knobs, optional `max_decode_dimension_px`. Apply defaults from the table where fields are omitted. Validate: non-negative weights; at least one of pHash/SSIM/histogram enabled for MVP; `threshold` in a sensible range (document clamp or error in README).
- Result: Loading a sample MVP JSON (see [Task.md](Task.md) §5 example) succeeds; unknown fields policy (ignore vs error) documented in README.
- Files/folders: `backend/src/config.rs`; sample `config.example.json` under `backend/` or repo root **MAY** be added.

**Context (read first)**

- [Task.md](Task.md) §5 (`config.json` contract, weight rule)

---

### 3. Scanner and vertex set (ingest rules) [DONE]

**Title:** Recursive directory scan, extension filter, symlink policy

**Description**

- What to do: Walk `--input` **recursively**; do **not** follow symlinks by default ([Task.md](Task.md) §3.8). Collect paths matching allowed extensions (at least those listed in §3.8). Skip unreadable paths with log/stderr line; never panic the whole run. Define stable ordering (e.g. sorted paths) for reproducible IDs.
- Result: List of candidate image paths suitable as graph vertices after hash attempt (§3.8 vertex rules).
- Files/folders: `backend/src/scanner.rs`; integrate from `main.rs`.

**Context (read first)**

- [Task.md](Task.md) §3.8 (operational defaults), §3.3 (vertices reference)

---

### 4. File hashing service [DONE]

**Title:** Byte-level digest per file for duplicate shortcut

**Description**

- What to do: For each scanned path with readable bytes, compute digest using configured algorithm (`hash_algorithm`, default per §5 table). Cache digests in memory keyed by path for the run. Expose equality check for clustering hash edges ([Task.md](Task.md) §3.1, §3.3).
- Result: Identical files produce identical digests; unreadable files skipped without crashing others.
- Files/folders: e.g. `backend/src/hash.rs` or inside `scanner`/loader pipeline.

**Context (read first)**

- [Task.md](Task.md) §3.1 (hash not in `final_score`), §3.3 (hash edges ignore threshold)

---

### 5. Image loader and decode failure handling [DONE]

**Title:** Decode images for metrics; tolerate failures per vertex rules

**Description**

- What to do: Decode raster formats supported by chosen crates; on decode failure, record failure reason and exclude decoded pixels for **weighted metrics**, but **keep vertex** if byte hash succeeded ([Task.md](Task.md) §3.8). Optionally honor `max_decode_dimension_px` by downscaling after decode.
- Result: For each vertex: optional `DecodedImage` / error state usable by similarity modules without panicking.
- Files/folders: `backend/src/image_loader.rs`; dependency choices documented per §3.7.

**Context (read first)**

- [Task.md](Task.md) §3.8 (vertex set + corrupt/decode), §3.7 (crate policy)

---

### 6. Similarity abstraction and `MetricResult` [DONE]

**Title:** Trait + normalization contract for metrics

**Description**

- What to do: Implement `SimilarityMetric::compute` returning `MetricResult { score, raw, valid }` as in appendix §A–§B. Ensure distance-like internals map to similarity `[0,1]` per appendix §B.
- Result: Shared type used by pHash/SSIM/histogram implementations; invalid pairs produce `valid: false`.
- Files/folders: `backend/src/similarity/mod.rs`, `backend/src/similarity/types.rs` (names flexible).

**Context (read first)**

- [Task_similarity_appendix.md](Task_similarity_appendix.md) §A, §B, §D

---

### 7. Metric: pHash [DONE]

**Title:** Perceptual hash and Hamming similarity

**Description**

- What to do: Implement pipeline appendix §C.2 (grayscale, resize, DCT block, median, 64-bit hash). Respect `enable_phash`, `phash_max_distance`, and weighting only via orchestration layer (task 9). Honor `enable_rotations` if already implemented for B (task 11) — or implement pHash without rotations first, then wire wrapper in task 11.
- Result: Valid `MetricResult` for pairs with both images decoded; invalid if decode missing.
- Files/folders: `backend/src/similarity/phash.rs` (or `features/phash.rs`).

**Context (read first)**

- [Task_similarity_appendix.md](Task_similarity_appendix.md) §C.2
- [Task.md](Task.md) §3.2 (methods table)

---

### 8. Metric: resize + SSIM [DONE]

**Title:** Shared resize pipeline and SSIM score

**Description**

- What to do: Implement appendix §C.3: optional alpha crop first (task 10 if enabled), resize to `resize_size`, consistent color space. SSIM score in `[0,1]`. Respect `enable_ssim`, `ssim_threshold` if used as a clamp/gate — **do not** confuse with global clustering `threshold` on `final_score` ([Task.md](Task.md) §5 notes).
- Result: Valid metric when both sides decode; invalid on failure/empty alpha if specified in appendix.
- Files/folders: `backend/src/similarity/ssim.rs`, shared resize helpers.

**Context (read first)**

- [Task_similarity_appendix.md](Task_similarity_appendix.md) §C.3, §C.4 (if alpha crop integrated here)

---

### 9. Metric: histogram [DONE]

**Title:** Histogram correlation (or Bhattacharyya) similarity

**Description**

- What to do: Implement appendix §C.5 with `hist_bins` / `hist_method` from config. Normalize to `[0,1]` similarity.
- Result: Valid metric when both images decode; configurable method.
- Files/folders: `backend/src/similarity/histogram.rs`

**Context (read first)**

- [Task_similarity_appendix.md](Task_similarity_appendix.md) §C.5

---

### 10. Preprocess: alpha crop (SHOULD for MVP polish) [DONE]

**Title:** Bounding box crop on alpha before SSIM/histogram/pHash inputs

**Description**

- What to do: Implement appendix §C.4 when `enable_alpha_crop` is true: detect alpha channel, crop to bbox above `alpha_threshold`, mark invalid when fully transparent.
- Result: Downstream metrics receive cropped buffers when enabled; no extra weighted term (per [Task.md](Task.md) §3.1 table).
- Files/folders: e.g. `backend/src/features/alpha_crop.rs` or inside `image_loader`.

**Context (read first)**

- [Task_similarity_appendix.md](Task_similarity_appendix.md) §C.4
- [Task.md](Task.md) §3.1 (preprocess vs score)

---

### 11. Optional: rotations wrapper (SHOULD) [DONE]

**Title:** Max over 0°/90°/180°/270° (and optional flip) for enabled metrics

**Description**

- What to do: Per [Task.md](Task.md) §3.1, for each enabled weighted metric, compute score at each rotation of image B (and flips if `enable_flip`), take max as that metric’s `score_i`. Document any metric that opts out in README.
- Result: Toggle in config changes behavior without duplicating metric math.
- Files/folders: `backend/src/similarity/rotations.rs` or wrapper in orchestration.

**Context (read first)**

- [Task.md](Task.md) §3.1 (rotations row), §5 `enable_rotations` / `enable_flip`
- [Task_similarity_appendix.md](Task_similarity_appendix.md) §C.6

---

### 12. `final_score` orchestration [DONE]

**Title:** Renormalized weights over valid enabled metrics per pair

**Description**

- What to do: Implement [Task.md](Task.md) §5 weight rule: for each pair, drop disabled and invalid metrics; renormalize weights over survivors; compute `final_score = sum w'_i * score_i`. If no metric participates, **no composite edge** (hash edges still allowed).
- Result: Deterministic scalar for each unordered pair where applicable.
- Files/folders: `backend/src/similarity/composite.rs` or `main.rs` orchestration module.

**Context (read first)**

- [Task.md](Task.md) §3.1 (pair flow), §5 (weight rule)
- [Task_similarity_appendix.md](Task_similarity_appendix.md) §D

---

### 13. Pairwise comparison loop and performance [DONE]

**Title:** Compare all unordered pairs with rayon; early exits where possible

**Description**

- What to do: Build edge set or adjacency structure: hash edges for equal digests without threshold; for remaining pairs compute `final_score` and add edge if `> threshold` (strict). Use `rayon` per §3.5. Apply cheap filters (e.g. dimension bucketing, hash inequality) before heavy work where safe and documented.
- Result: Correct graph for clustering on test folders; reasonable time for ≥1000 images on default settings **or** documented limitations.
- Files/folders: `backend/src/similarity/pairwise.rs` or `main.rs`; `rayon` in `Cargo.toml`.

**Context (read first)**

- [Task.md](Task.md) §3.3 (clustering), §3.5 (performance)

---

### 14. Clustering — connected components [DONE]

**Title:** Components as output groups with stable IDs

**Description**

- What to do: Union-find or BFS/DFS on undirected graph from task 13. Assign `groups[].id` (e.g. incrementing stable under sorted vertex order per task 3). Implement singleton policy chosen in README ([Task.md](Task.md) §3.3): either emit every singleton group or omit singletons — pick once and implement consistently.
- Result: `groups` array matching clustering semantics.
- Files/folders: `backend/src/clustering.rs`

**Context (read first)**

- [Task.md](Task.md) §3.3

---

### 15. `groups[].score` computation [DONE]

**Title:** Max pairwise `S` over defined pairs per §6

**Description**

- What to do: Implement [Task.md](Task.md) §6 exactly: omit `score` for singletons (MVP MUST omit); for `|G|≥2`, max over pairs where `S` is defined (hash-equal ⇒ 1.0; else valid `final_score`). Exclude pairs with undefined composite; omit field if no qualifying pairs.
- Result: Output JSON matches semantic acceptance for reporting.
- Files/folders: post-processing pass after clustering in `backend/src/` (e.g. `report.rs`).

**Context (read first)**

- [Task.md](Task.md) §6 (`result.json`, `groups[].score`)

---

### 16. Write `result.json` and harden CLI exit behavior [DONE]

**Title:** Serialize results; absolute paths; exit codes

**Description**

- What to do: Emit JSON matching §6 structure; canonical **absolute** paths in `groups[].images` per §3.8. Exit `0` on success after atomic write (write temp + rename **SHOULD**). Non-zero on fatal errors with stderr message §3.9.
- Result: Flutter can consume output reliably.
- Files/folders: `backend/src/output.rs` or inline in `main.rs`; integration tests optional.

**Context (read first)**

- [Task.md](Task.md) §6, §3.8 (paths), §3.9

---

### 17. Flutter project scaffold (desktop) [DONE]

**Title:** Create `frontend/` Flutter desktop app shell

**Description**

- What to do: `flutter create` (or equivalent) targeting desktop ([Task.md](Task.md) §4 layout). Minimal routing: Settings / Scan / Results placeholders.
- Result: App launches on macOS; folder structure aligns with §4.3 suggestion.
- Files/folders: `frontend/` tree (`lib/main.dart`, `lib/screens/`, etc.)

**Context (read first)**

- [Task.md](Task.md) §4 (Flutter requirements), §4.3 layout

---

### 18. Flutter — settings UI and `config.json` emission [DONE]

**Title:** Mirrors MVP config fields and canonical example

**Description**

- What to do: Widgets for toggles (pHash/SSIM/histogram/optional alpha/rotations), threshold slider/field, weights inputs summing hints (Flutter **SHOULD** persist weights as user enters; backend renormalizes per pair anyway per §5). Write JSON file to temp path before each scan matching §5 MVP example shape where applicable.
- Result: Generated file loads in Rust `Config` from task 2.
- Files/folders: `frontend/lib/screens/settings_*`, `frontend/lib/models/config.dart`

**Context (read first)**

- [Task.md](Task.md) §5

---

### 19. Flutter — `backend_runner` service [DONE]

**Title:** Spawn Rust CLI; stream stdout/stderr; capture exit code

**Description**

- What to do: Implement §4.4: locate binary (bundled resource path vs PATH vs sibling `backend/target/release/` — document in README §7). Pass `--input`, `--output`, `--config`. Surface stderr tail on failure ([Task.md](Task.md) §3.9 UI MUST).
- Result: Scan button runs backend end-to-end on developer machine (paths documented).
- Files/folders: `frontend/lib/services/backend_runner.dart`

**Context (read first)**

- [Task.md](Task.md) §2 (invocation), §4.4, §7 item 4

---

### 20. Flutter — parse results and results UI [DONE]

**Title:** Load `result.json`; grid with lazy thumbnails

**Description**

- What to do: Implement §4 MUST: parse groups (§6); show group list with counts; thumbnail grid with lazy loading; SHOULD items as time permits (zoom, side-by-side, filters). Handle missing `score` on singletons per §6.
- Result: Visual verification of clustering on sample folders.
- Files/folders: `frontend/lib/services/json_parser.dart`, `frontend/lib/screens/results_*`, widgets under `frontend/lib/widgets/`

**Context (read first)**

- [Task.md](Task.md) §4 (MVP/SHOULD), §6

---

### 21. README and acceptance checklist [DONE]

**Title:** Document build, defaults, limits, binary discovery

**Description**

- What to do: Fulfill [Task.md](Task.md) §7: MVP vs Phase 2; platform build commands; how UI finds binary; example CLI invocation; §3.8 defaults (extensions, recursion, symlinks); §3.9 behavior; singleton policy; chosen crates ([Task.md](Task.md) §3.7). Walk through §8 acceptance criteria and tick boxes mentally or in doc.
- Result: New contributor can build and run MVP without reading source first.
- Files/folders: `README.md` (repo root)

**Context (read first)**

- [Task.md](Task.md) §7, §8, §0

---

## Phase 2 — Optional extensions

Complete Phase 1 tasks **before** starting these unless explicitly prioritized.

### 22. Metric: ORB + config wiring

**Title:** Feature matching metric and weights

**Description**

- What to do: Implement appendix §C.7; extend config §5 Phase 2 keys; include ORB in renormalization when enabled and valid.
- Result: Improved robustness on cropped/rotated pairs where applicable; README warns about weak cases per appendix.
- Files/folders: `backend/src/similarity/orb.rs`, `config.rs`, Flutter settings.

**Context (read first)**

- [Task_similarity_appendix.md](Task_similarity_appendix.md) §C.7
- [Task.md](Task.md) §0 Phase 2, §3.2 ORB row

---

### 23. Game-dev heuristics (normal maps, masks, tileability)

**Title:** Appendix §C.8 detection and preprocessing hooks

**Description**

- What to do: Filename/heuristic classification and specialized preprocessing paths before metrics; optional cyclic shifts for tileables per appendix.
- Result: Better clustering on specialized asset types; heavily README-documented assumptions.
- Files/folders: `backend/src/features/game_dev.rs` (or split modules).

**Context (read first)**

- [Task_similarity_appendix.md](Task_similarity_appendix.md) §C.8

---

### 24. Flutter — destructive actions and richer UX

**Title:** Delete/move files with safeguards; drag-drop; export report

**Description**

- What to do: Only after explicit Phase 2 approval per [Task.md](Task.md) §0 and §4 MAY: deletion requires confirmation + undo/trash policy; drag-drop folder selection; export CSV/HTML summary if desired.
- Result: Safer production UX beyond MVP browse/compare.

**Context (read first)**

- [Task.md](Task.md) §0 Phase 2, §4 MAY

---

### 25. Optional machine-readable progress (`--progress-json`)

**Title:** Stable stdout/stderr protocol for Flutter progress bars

**Description**

- What to do: If needed beyond MVP, add optional CLI flag contradicting §3.9 “no stable stdout schema”; define NDJSON or similar **versioned** lines; update Flutter runner.
- Result: Deterministic UI progress without breaking default human stdout.

**Context (read first)**

- [Task.md](Task.md) §3.9 (current MVP constraint)
