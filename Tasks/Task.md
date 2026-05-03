# AI Agent Task: Similar Texture Finder

## Document map

| File | Use |
|------|-----|
| [Task.md](Task.md) | Scope, contracts (CLI, config, results), MVP vs later, acceptance — **attach this by default.** |
| [Task_similarity_appendix.md](Task_similarity_appendix.md) | Metric pipelines, trait/normalization detail — **attach only when implementing or tuning similarity.** |

**Agents:** Plan wiring (Flutter runner, JSON I/O, clustering) from `Task.md` alone; open the appendix when coding `features/` / `similarity/` or adjusting metric behavior.

---

## 0) MVP vs Phase 2 (implementation order)

**Phase 1 (MVP) — MUST ship before extras**

1. Rust CLI: scan → pairwise similarity → clustering → `result.json`.
2. Weighted `final_score` with **renormalized** weights (`§5`) for **pHash + SSIM + histogram** only.
3. File hash shortcut (`§3.3`) for exact duplicates.
4. Flutter: folder picker, `config.json` generation, run CLI, parse results, grid + lazy thumbnails.
5. Error-tolerant ingest (`§3.8`): skip bad files, non-zero exit on fatal errors (`§3.9`).
6. README (`§7`) documents defaults in `§3.8–§3.9`.

**Phase 2 — MAY**

- ORB, normal-map / mask heuristics, tileable-shift tricks (`Task_similarity_appendix.md`).
- UI: delete files (with confirmation + explicit undo/trash policy), drag-drop, export reports.
- Filter by “texture type” **only after** defining detection rules (filename/extension/heuristic); otherwise omit.

---

## 1) Goal

**MUST (MVP):** Cross-platform standalone desktop app to find similar textures in a user-chosen folder.

- **Rust:** scan images, compute similarities, cluster, write JSON.
- **Flutter:** folder pick, settings, run CLI, show groups and previews.

**Platforms:** macOS first **MUST**; Windows/Linux **SHOULD** use same codebase paths.

---

## 2) Architecture

### Components

| Part | Responsibility |
|------|----------------|
| Rust CLI | Scan, compare, score, cluster, export JSON |
| Flutter desktop UI | Pick folder, emit `config.json`, run CLI, parse results, display UX |

### Integration contract (**MUST**)

- Transport: **CLI + JSON only** — **no FFI** for MVP.
- Invocation shape:

  `texture_tool --input <folder> --output <result.json> --config <config.json> [--threads <n>]`

- Rust writes **complete** `result.json`; Flutter reads it after exit (streaming stdout **MAY** be human-only unless you later define a machine progress format).

---

## 3) Backend (Rust)

### 3.1 Composite pipeline (**MUST** understand before coding)

What contributes to **`final_score`** vs preprocessing-only:

| Step | Role |
|------|------|
| Byte-level **file hash** | **Not** part of `final_score`. Used only for **exact duplicate shortcut** `S_hash = 1.0` when digests match (`§3.3`). |
| Decode → raster | Required before **weighted metrics** (pHash, SSIM, histogram, ORB). |
| **Alpha crop** | Preprocess only (no weighted term). |
| **Rotations** (optional) | For each unordered pair and each **enabled** weighted metric run in Phase 1/2: compute metric at `0°/90°/180°/270°` on **B** (and flips if `enable_flip`), take **max**; that metric’s `score_i` feeds `final_score`. **MUST** document if any metric opts out. |
| pHash, SSIM, histogram, (ORB) | **Only** sources of `final_score`; each returns `score_i ∈ [0,1]` or `invalid` (see appendix). |

Ordered flow for each pair `(A, B)` **after** both pass ingest (`§3.8`):

1. Compare **file hashes**; if equal, **clustering edge** exists regardless of `threshold` (`§3.3`).
2. If hashes differ (or hash not used): compute **`final_score(A,B)`** from enabled metrics only (`§5`, appendix §D).
3. If `final_score(A,B) > threshold` (strict `>`), add **composite** clustering edge (`§3.3`).

If all enabled metrics are **invalid** for a pair, **no composite edge** is possible from that pair; hash edges can still connect components.

### 3.2 Methods overview

Detailed pipelines live in the appendix. Here: role in **`final_score`** vs preprocessing-only.

| Method | Role | MVP | Primary config keys (names) |
|--------|------|-----|-------------------------------|
| File hash (`MD5`/`SHA1`/`SHA256`/…) | Shortcut `1.0` if equal — **not** a weighted term | MUST | `hash_algorithm` (see `§5`) |
| pHash | Weighted metric | MUST | `enable_phash`, `phash_max_distance`, weight |
| Resize + SSIM | Weighted metric | MUST | `enable_ssim`, `resize_size`, `ssim_threshold`, weight |
| Histogram | Weighted metric | MUST | `enable_histogram`, `hist_bins`, `hist_method`, weight |
| Alpha crop | Preprocess for SSIM/histogram/etc. when enabled | SHOULD | `enable_alpha_crop`, `alpha_threshold` |
| Rotations (`0/90/180/270`) | Wrapper max-over-rotations for chosen metrics | SHOULD | `enable_rotations`, optional `enable_flip` |
| ORB | Weighted metric | MAY (Phase 2) | `enable_orb`, `orb_*`, weight |
| Normal maps / grayscale rules | Detection + channel handling | MAY | naming / heuristics (appendix) |

**ORB / game-dev extras:** implement **after** MVP passes acceptance unless explicitly instructed otherwise.

### 3.3 Clustering (**MUST**)

- **Vertices:** eligible images discovered under `--input` per ingest rules (`§3.8`).
- **Undirected edge** between distinct vertices `A` and `B` iff **either:**
  - **Hash (ignores `threshold`):** `file_hash(A) == file_hash(B)` ⇒ edge **always** (exact duplicates).
  - **Composite:** hashes differ **and** `final_score(A,B) > threshold` (strict `>`).
- **Components:** connected components are output **groups**.
- **`singletons`:** **MUST** emit single-image groups **or** omit them — pick **one** in README and keep stable across releases.

### 3.4 CLI (**MUST**)

| Argument | Meaning |
|----------|---------|
| `--input <folder>` | Root to scan |
| `--output <file.json>` | Write results here |
| `--config <config.json>` | Analysis toggles + thresholds + weights |
| `--threads <n>` | Worker threads (**SHOULD**; default sensible on crate side) |

Binary/crate name **SHOULD** match whatever Flutter invokes; align `Cargo.toml` `[[bin]]` name with docs.

### 3.5 Performance (**SHOULD**)

- Use `rayon` (or equivalent) for parallelism.
- Reduce `(n²)` work via hashing / cheap filters before expensive metrics.
- Avoid loading full decoded images for all pairs simultaneously.

### 3.6 Suggested layout

```text
backend/
  src/
    main.rs
    scanner.rs
    image_loader.rs
    features/
    similarity/
    clustering.rs
    config.rs
```

### 3.7 Rust crate policy (**SHOULD**)

- Prefer **maintained**, permissively-licensed crates; record pinned choices and rationale in README (e.g. `image`, `rayon`; similarity primitives left to implementer).
- If choosing heavyweight CV bindings (e.g. OpenCV), **MUST** document platform install steps per OS.

### 3.8 Operational defaults (**MUST** document in README; implement consistently)

| Topic | Normative default (unless README documents otherwise) |
|-------|------------------------------------------------------|
| **Scan depth** | **Recursive**: include nested subdirectories under `--input`. |
| **Symlinks** | **Do not follow** symlinks (avoids cycles); ignore broken links with log line. |
| **Extensions** | Only decode known raster extensions; MVP **SHOULD** include at least `png`, `jpg`, `jpeg`, `webp`, `bmp`, `gif`, `tiff`, `tif`. Unknown extensions **skipped** with optional debug log. |
| **Vertex set** | Every file under `--input` matching extension rules with **readable bytes** for hashing **MUST** be a vertex (pixel decode may still fail). Decode failure **MUST NOT** drop the vertex if hash was computed — it can still form **hash-only** edges and singletons. Files unreadable at byte level **MUST** be skipped (log + no vertex). |
| **Unreadable / corrupt / decode failures** | **MUST NOT** panic entire run. Emit stderr/log with path + reason. Skip weighted metrics for failed decode; still use byte hash when possible. |
| **Large images** | **SHOULD** support optional `max_decode_dimension_px` in `config.json` (downscale before metrics); if present, document. |
| **Output paths** | **MUST** use **absolute, normalized** paths in `groups[].images` (e.g. canonicalize once at write). |

### 3.9 Process / CLI I/O (**MUST**)

| Rule | Detail |
|------|--------|
| **Exit code** | `0` = success (wrote valid `result.json`). Non-zero = failed; partial output **undefined** unless documented. |
| **stderr** | Human-readable errors; UI **MUST** show failure + stderr tail. |
| **stdout** | Human progress text **MAY**; **no stable machine schema in MVP** unless you add optional `--progress-json` later. |

---

## 4) Frontend (Flutter)

### MUST (MVP)

- Native folder picker; persist last path **MAY**.
- Settings UI mapped to **`config.json`** (enable flags, threshold, weights — MVP subset in §5).
- **Scan** runs Rust process; show spinner + stdout tail/log.
- Parse `result.json`; group list + thumbnail grid with lazy loading.
- Non-destructive actions **SHOULD:** open in Finder/Explorer; side-by-side compare **SHOULD**.

### SHOULD

- Zoom / preview detail pane.
- Filters by group size or score range.

### MAY / Phase 2

- Destructive actions (**delete files**) — require confirmation + undo/trash policy if added.
- Filter “by texture type” — define detection rule first or omit.
- Drag-and-drop folder; export HTML/CSV report; save full UI state.

### Layout (**SHOULD**)

```text
frontend/
  lib/
    main.dart
    screens/
    widgets/
    models/
    services/
      backend_runner.dart
      json_parser.dart
```

`backend_runner.dart` **MUST:** spawn process, stream stdout/stderr, await exit code, surface failures.

---

## 5) `config.json` contract

Full metric knobs narrative: [Task_similarity_appendix.md](Task_similarity_appendix.md).

### Weight rule (**MUST**)

- Config stores nonnegative weights per scoring channel (`phash`, `ssim`, `histogram`, `orb`).
- For each **pair**, consider only metrics that are **enabled** and produced **`valid`** output for that pair.
- **Effective weight** `w'_i = w_i / Σ w_j` over participating channels only.
- **`final_score = Σ w'_i * score_i`** over those channels.
- If **no** channel participates (all invalid/disabled), **do not** create a composite edge — hash shortcut may still apply.

JSON Schema **MAY** be added later; field list below is authoritative for MVP naming.

### Field list

| Field | Type | MVP | Default (if omitted) | Notes |
|-------|------|-----|----------------------|-------|
| `enable_phash` | bool | MUST | — | Required in MVP config |
| `enable_ssim` | bool | MUST | — | |
| `enable_histogram` | bool | MUST | — | |
| `enable_orb` | bool | MAY | `false` | Phase 2; omit from MVP example |
| `enable_alpha_crop` | bool | SHOULD | `false` | |
| `enable_rotations` | bool | SHOULD | `false` | |
| `enable_flip` | bool | MAY | `false` | With rotations; appendix |
| `threshold` | number | MUST | — | Composite edge if `final_score > threshold` only |
| `weights` | object | MUST | — | Keys below; values ≥ 0 |
| `weights.phash` | number | MUST | — | Renormalize over **enabled valid** metrics per pair (`§5` Weight rule) |
| `weights.ssim` | number | MUST | — | |
| `weights.histogram` | number | MUST | — | |
| `weights.orb` | number | MAY | — | Phase 2 |
| `hash_algorithm` | string | SHOULD | `"sha256"` | e.g. `md5`, `sha1`, `sha256` — document supported set |
| `phash_max_distance` | int | SHOULD | `10` | Appendix C.2 |
| `ssim_threshold` | number | SHOULD | `0.9` | Filtering/tuning; edge still uses global `threshold` on `final_score` |
| `resize_size` | int | SHOULD | `256` | SSIM / shared resize (document) |
| `hist_bins` | int | SHOULD | `512` | Or triple product per appendix |
| `hist_method` | string | SHOULD | `"correlation"` | e.g. `correlation`, `bhattacharyya` |
| `alpha_threshold` | number | SHOULD | `0.05` | Alpha crop mask cutoff |
| `orb_max_features` | int | MAY | — | Phase 2 |
| `orb_match_threshold` | number | MAY | — | Phase 2 |
| `max_decode_dimension_px` | int | MAY | unlimited | Downscale gate |

Extended keys **SHOULD** match [Task_similarity_appendix.md](Task_similarity_appendix.md).

### MVP example (**canonical baseline**)

```json
{
  "enable_phash": true,
  "enable_ssim": true,
  "enable_histogram": true,
  "enable_alpha_crop": false,
  "enable_rotations": false,
  "threshold": 0.85,
  "weights": {
    "phash": 0.35,
    "ssim": 0.45,
    "histogram": 0.2
  }
}
```

---

## 6) `result.json` contract

### Field list

| Field | Type | Meaning |
|-------|------|---------|
| `groups` | array | Cluster output |
| `groups[].id` | int | Stable ID within file |
| `groups[].images` | string[] | Paths — **SHOULD** be absolute or consistently relative + documented |
| `groups[].score` | number \| omitted | See below |

### `groups[].score` definition (**MUST**)

Define **pairwise similarity** `S(A,B)` for any two **distinct ingested** images (same formulas as clustering):

- If `file_hash(A) == file_hash(B)` ⇒ `S(A,B) = 1.0`.
- Else ⇒ `S(A,B) = final_score(A,B)` (computed per `§5` even if **no edge** existed between them — e.g. they meet only via a third image).

Then **`groups[].score`** for a group **G**:

- If `|G| < 2`: **omit** `score` (MVP **MUST** omit; no `null` required).
- If `|G| ≥ 2`: `score = max` of `S(image_i, image_j)` over unordered pairs `{i,j}` for which **`S` is defined**:
  - **Defined** if `file_hash(i)==file_hash(j)` **or** `final_score(i,j)` was computable with at least one participating metric (`§5`).
  - If hashes differ and `final_score` is **undefined** (all metrics invalid), **exclude** that pair from the max.
- If **no** pair in `G` has defined `S` while `|G| ≥ 2`, **omit** `score`.

**Note:** Worst-case cost is **O(|G|²)** metric evaluations per group; acceptable for typical small components. **MAY** document a future optional approximate score (e.g. max over spanning tree edges) if large cliques appear.

### Example

```json
{
  "groups": [
    {
      "id": 1,
      "score": 0.93,
      "images": ["/abs/path/a.png", "/abs/path/b.png"]
    }
  ]
}
```

---

## 7) README (**MUST** deliver eventually)

Include at minimum:

1. What the tool does (MVP vs optional metrics) + **Phase checklist** alignment (`§0`).
2. How to build backend (`cargo build --release` from `backend/`).
3. How to build Flutter app (`flutter build macos` / platform targets).
4. How UI locates and invokes the Rust binary; exit code + stderr expectations (`§3.9`).
5. Example invocation + sample config/output pointers.
6. Limits: supported extensions, symlink policy, recursion, max image size/downscale (`§3.8`), perf, metric weaknesses.

JSON Schema **MAY** be added in a later revision; tables in `§5–§6` remain source of truth until then.

---

## 8) Acceptance criteria (**MVP**)

- Finds exact duplicates (hash) and plausible near-duplicates via configured metrics.
- Grouping matches §3.3 graph rule.
- Handles **≥ 1000** images without exhausting memory under sane settings.
- Skips/logs corrupt inputs instead of crashing whole run.
- Desktop UI runs scan end-to-end against real CLI.

---

## 9) Nice-to-have

Backend: caching intermediates; structured logging.

UI: drag-drop folder; remembered settings; export report.

---

## 10) Deliverables (**MUST**)

1. Rust CLI analyzer (`backend/`).
2. Flutter desktop app (`frontend/`).
3. CLI + JSON integration per §§2–6.
4. README per §7.

---

## Reference

Metric trait, normalization, per-method pipelines, ORB/game-dev detail: **[Task_similarity_appendix.md](Task_similarity_appendix.md)**.
