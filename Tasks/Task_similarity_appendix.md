# Similar Texture Finder — similarity methods appendix

Normative scope, CLI contract, clustering rules, MVP tags, and config/output field lists live in [Task.md](Task.md). Use **this file** only when implementing or tuning similarity metrics.

---

## A. Common interface for all methods

```rust
trait SimilarityMetric {
    fn compute(&self, img_a: &ImageData, img_b: &ImageData) -> MetricResult;
}

struct MetricResult {
    score: f32, // normalized [0.0 - 1.0]
    raw: f32,   // raw metric value
    valid: bool // whether computation was successful
}
```

## B. Normalization rules

- Every method maps to `0.0` = completely different, `1.0` = identical.
- Distance metrics ("lower is better") must be inverted into similarity.

## C. Method specs

### C.1 File hash (`MD5` / `SHA1` / `SHA256` / …)

**Purpose:** exact file duplicates.

**Implementation:**

- Compute file hash; `score = 1.0` if equal else `0.0`.

**Notes:** early filter; cache hash once per file. **Not** combined into `final_score` in [Task.md](Task.md); used for the clustering hash shortcut only.

### C.2 pHash (perceptual hash)

**Purpose:** visually similar images after resize/compression/minor edits.

**Pipeline:** grayscale → resize `32×32` → DCT → `8×8` low-frequency block → median → `64-bit` hash.

**Comparison:** `distance = hamming(hash_a, hash_b)`; `score = 1.0 - (distance / 64.0)`.

**Parameters:** `phash_max_distance` (e.g. 10); if exceeded → `score = 0`.

### C.3 Resize + SSIM

**Purpose:** near-identical appearance.

**Pipeline:** alpha crop if enabled → resize (`128×128` or `256×256`) → single color space (`RGB` or grayscale).

**Score:** `score = ssim_value` in `[0..1]`.

**Parameters:** `ssim_threshold` (e.g. `0.9`), `resize_size`.

### C.4 Alpha crop

**Purpose:** ignore transparent borders (sprites/UI).

**Pipeline:** if alpha exists, bounding box where `alpha > threshold` (e.g. 5%); crop.

**Edge cases:** no alpha → skip; fully transparent → invalid.

**Parameters:** `alpha_threshold` (`0.0–1.0`).

### C.5 Histogram comparison

**Purpose:** color distribution similarity.

**Pipeline:** resize (e.g. `128×128`) → histogram `RGB` (e.g. `8×8×8` bins) or grayscale.

**Metric:** correlation `score = correlation(hist_a, hist_b)`, or Bhattacharyya distance mapped to score.

**Parameters:** `hist_bins`, `hist_method`.

### C.6 Rotations

**Purpose:** same texture under rotation.

**Pipeline:** for image B try `0°/90°/180°/270°`; run underlying metric; `score = max(...)`.

**Parameters:** `enable_rotations`, `enable_flip` (optional).

**Normative alignment:** when enabled, apply the max-over-rotations rule per **weighted** metric (pHash, SSIM, histogram, ORB when on) unless README documents an exception.

### C.7 ORB (feature matching)

**Purpose:** robustness under crop/scale/rotation.

**Pipeline:** grayscale → ORB keypoints/descriptors → match (`BFMatcher` / Hamming).

**Score:** `score = good_matches / max(keypoints_a, keypoints_b)`.

**Limitations:** weak on smooth textures, gradients, flat UI assets.

**Parameters:** `orb_max_features`, `orb_match_threshold`.

### C.8 Game-dev specifics

**Normal maps:** detect via filename (`_n`, `normal`) and/or channel stats; optionally ignore blue (`Z`) or normalize `(x,y,z)` to unit vector; compare `X,Y` via SSIM/histogram.

**Grayscale / masks:** force grayscale; emphasize structure via SSIM + single-channel histogram.

**Tileable textures (optional):** cyclic shifts; compare multiple offsets.

## D. Final score integration

Each contributing metric returns `score_i ∈ [0, 1]`.

**Weighted combination:** `final_score = Σ (weight_i * score_i)` over metrics that are **enabled** and **valid** for that pair.

**Rules:**

- Renormalize weights over metrics that participate in that pair (see [Task.md](Task.md)).
- Disabled metrics do not participate.
- Invalid metric results are excluded from the sum (renormalize remaining weights).

## E. Principles

- Combination of methods, not a single algorithm.
- Metrics stay independent and configurable.
- Unified `[0, 1]` scale for comparable outputs.
- Performance: cache aggressively where useful.
