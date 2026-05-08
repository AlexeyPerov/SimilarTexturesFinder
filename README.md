# Similar Textures

Desktop MVP: **Rust** CLI (`texture_tool`) clusters similar textures and writes `result.json`; **Flutter** UI picks a folder, edits settings, runs the CLI, and shows groups with thumbnails.

Normative behavior: [Tasks/Task.md](Tasks/Task.md), [Tasks/Task_similarity_appendix.md](Tasks/Task_similarity_appendix.md), [Tasks/Execution_Plan.md](Tasks/Execution_Plan.md).

## MVP vs Phase 2

See [Tasks/Task.md](Tasks/Task.md) **§0**. Phase 1 is the pHash + SSIM + histogram pipeline, file-hash duplicates, Flutter folder picker + scan + results. Phase 2 (ORB, game-dev heuristics, richer UI) is optional unless you extend the project.

## Backend (Rust)

```bash
cd backend
cargo build --release
```

Binary: `backend/target/release/texture_tool` (`.exe` on Windows).

Example CLI ([Tasks/Task.md](Tasks/Task.md) §2):

```bash
./target/release/texture_tool \
  --input /path/to/folder \
  --output /tmp/result.json \
  --config backend/config.example.json \
  --threads 8
```

- **Exit `0`** only after a successful atomic write of `result.json`. Failures: **non-zero** + **stderr** for humans ([Tasks/Task.md](Tasks/Task.md) §3.9). The Flutter app surfaces a **stderr tail** on failure.
- **`groups[].images`** are **absolute**, canonical paths ([Tasks/Task.md](Tasks/Task.md) §3.8).

### Rust dependencies (pinned in repo)

Main crates: `clap`, `image`, `serde` / `serde_json`, `walkdir`, `md-5`, `sha1`, `sha2`, `digest`, `rustdct`, `rayon`, `tempfile`, `path` (via `image` / std). See [backend/Cargo.toml](backend/Cargo.toml).

## Flutter desktop app

Requires Flutter with **desktop** enabled (e.g. macOS: `flutter config --enable-macos-desktop`).

```bash
cd frontend
flutter pub get
flutter run -d macos
```

Release build (macOS):

```bash
flutter build macos
```

App layout: [Tasks/Task.md](Tasks/Task.md) §4.3 — `lib/main.dart`, `screens/`, `widgets/`, `models/`, `services/`.

### Diagnostics and macOS crashes

- The app appends timestamped lines to **`similar_textures_diagnostic.log`** under Flutter’s **Application Support** directory for this app (same place as `user_preferences.json`). Scan phases, Dart `FlutterError`s, and uncaught async errors are logged there when possible.
- **While developing:** run `flutter run -d macos` and watch the terminal; optional: **Xcode → Open Developer Tool → Console** filtered by the app name.
- **Hard native crashes:** open **Console.app** (Crash Reports) or inspect **`~/Library/Logs/DiagnosticReports/`** for `.ips` files mentioning your process or bundle id.

**macOS packaging:** App Sandbox is **disabled** for the desktop target so the UI can spawn `texture_tool` and it can read normal user folders. Building the `.app` runs a **Copy texture_tool** phase that copies `backend/target/release/texture_tool` (or `debug`) into `Contents/MacOS/` when it exists — run `cargo build --release` in `backend/` before `flutter build macos` so the bundle includes the binary.

### How the UI finds `texture_tool` ([Tasks/Task.md](Tasks/Task.md) §4 + §7)

Resolution order (see [frontend/lib/services/backend_runner.dart](frontend/lib/services/backend_runner.dart)); the same order is summarized in the app under **Settings → Backend (texture_tool)**, where you can also **choose the executable** with a file picker (path is stored for this app on the device).

1. **Optional path** saved in Settings (if the file still exists).
2. **Bundled** `texture_tool` in the `.app` under `Contents/MacOS/` (when the Xcode copy phase found a built Rust binary).
3. Environment variable **`TEXTURE_TOOL_PATH`** (full path to the executable).
4. **Development:** `../backend/target/release/texture_tool` then `../backend/target/debug/texture_tool`, relative to the **current working directory** (typically the `frontend/` directory when you run `flutter run` from there).
5. **`texture_tool` on PATH** (`which` / `where`).

Set `TEXTURE_TOOL_PATH` for custom installs or when the CWD is not next to `backend/`. For a built macOS `.app`, prefer the bundled helper or **Settings**.

## Operational defaults ([Tasks/Task.md](Tasks/Task.md) §3.8)

| Topic | Behavior |
|--------|-----------|
| Scan | Recursive under `--input` |
| Symlinks | Not followed; broken links skipped with log |
| Extensions | Raster types: png, jpg, jpeg, webp, bmp, gif, tiff, tif |
| Vertices | Readable bytes for hashing required; decode may fail but vertex can remain for hash edges |
| Large images | Optional `max_decode_dimension_px` in config |
| Output paths | Absolute normalized paths in JSON |

## Config and results

- Sample config: [backend/config.example.json](backend/config.example.json) (matches [Tasks/Task.md](Tasks/Task.md) §5 field names).
- Results schema: [Tasks/Task.md](Tasks/Task.md) §6 (`groups[].id`, `images`, optional `score`).

## Clustering / singletons

Every ingested image appears in **exactly one** group. **Singleton** groups (size 1) are included; **`score` is omitted** for singletons or when no pair in the group has a defined similarity ([Tasks/Task.md](Tasks/Task.md) §6).

## Metrics (short)

- **pHash**, **SSIM**, **histogram**: weighted `final_score` with per-pair renormalized weights ([Tasks/Task.md](Tasks/Task.md) §5).
- **File hash**: exact duplicate shortcut only; not part of `final_score`.
- **SSIM gate**: `ssim_threshold` can mark SSIM **invalid** for that pair (not the same as global clustering `threshold`).
- **Histogram**: `hist_bins` maps to per-channel bins via cube root (e.g. 512 → 8³); `hist_method`: `correlation` or `bhattacharyya`.
- **Rotations / flip**: optional max-over-transforms of image B for all three MVP metrics ([Tasks/Task_similarity_appendix.md](Tasks/Task_similarity_appendix.md) §C.6).

## Acceptance checklist (MVP, [Tasks/Task.md](Tasks/Task.md) §8)

Use this as a manual QA list:

- [ ] Exact duplicates cluster via file hash; near-duplicates configurable via pHash/SSIM/histogram.
- [ ] Grouping follows undirected graph rule: hash edges ignore `threshold`; composite edges when `final_score > threshold` ([Tasks/Task.md](Tasks/Task.md) §3.3).
- [ ] Runs on the order of **≥ 1000** images without exhausting memory under reasonable settings (depends on machine; pairwise is O(n²)).
- [ ] Corrupt/unreadable images are skipped or logged; the run does not panic.
- [ ] Desktop UI: pick folder, run scan, see **stderr** on failure, load **`result.json`** and browse thumbnails.

## Phase 2 ideas

ORB, normal-map heuristics, tileable shifts, destructive file ops — see [Tasks/Task.md](Tasks/Task.md) §0 and [Tasks/Execution_Plan.md](Tasks/Execution_Plan.md) after task 21.
