use serde::{Deserialize, Serialize};
use similar_textures_core::config::Config;
use similar_textures_core::{
    image_loader::ImageData, output::read_result_json, run_scan, write_result_csv, write_result_json,
    ScanCallbacks, ScanEvent, ScanOutcome, ScanRequest, ScanResult, ScanStatus,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Emitter, Manager, State};

const SETTINGS_FILE: &str = "settings.json";
const SETTINGS_SCHEMA_VERSION: u32 = 1;
const UI_STATE_FILE: &str = "ui-state.json";
const UI_STATE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct StartScanRequest {
    input_dir: String,
    threads: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ExportRequest {
    target_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ScanStatusResponse {
    state: String,
    active_scan_id: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct LastResultResponse {
    has_result: bool,
    result: Option<ScanResult>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct StartScanResponse {
    scan_id: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ExportResponse {
    path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct UiStateResponse {
    last_input_dir: Option<String>,
    theme: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct StoredUiStateV1 {
    version: u32,
    last_input_dir: Option<String>,
    #[serde(default)]
    theme: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct CountImagesResponse {
    image_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct LoadResultRequest {
    source_path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct LoadResultResponse {
    group_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ImagePreviewRequest {
    path: String,
    max_px: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ImagePreviewResponse {
    path: String,
}

const DEFAULT_PREVIEW_MAX_PX: u32 = 256;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct StoredSettingsV1 {
    version: u32,
    config: Config,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
enum StoredSettingsFile {
    V1(StoredSettingsV1),
    Legacy(Config),
}

struct AppSlots {
    status: Mutex<ScanStatusValue>,
    active_scan_id: AtomicU64,
    scan_sequence: AtomicU64,
    cancel_flag: Arc<AtomicBool>,
    last_result: Mutex<Option<ScanResult>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ScanStatusValue {
    Idle,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl ScanStatusValue {
    fn as_str(self) -> &'static str {
        match self {
            ScanStatusValue::Idle => "idle",
            ScanStatusValue::Running => "running",
            ScanStatusValue::Completed => "completed",
            ScanStatusValue::Failed => "failed",
            ScanStatusValue::Cancelled => "cancelled",
        }
    }
}

impl Default for AppSlots {
    fn default() -> Self {
        Self {
            status: Mutex::new(ScanStatusValue::Idle),
            active_scan_id: AtomicU64::new(0),
            scan_sequence: AtomicU64::new(0),
            cancel_flag: Arc::new(AtomicBool::new(false)),
            last_result: Mutex::new(None),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ScanLogEventPayload {
    scan_id: u64,
    line: String,
    level: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ScanProgressEventPayload {
    scan_id: u64,
    phase: String,
    processed: Option<usize>,
    total: Option<usize>,
    vertices: Option<usize>,
    groups: Option<usize>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ScanFinishedEventPayload {
    scan_id: u64,
    status: String,
    message: Option<String>,
}

fn app_config_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app_config_dir(app)?.join(SETTINGS_FILE))
}

fn ui_state_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app_config_dir(app)?.join(UI_STATE_FILE))
}

fn normalize_theme(theme: Option<String>) -> String {
    match theme.as_deref() {
        Some("light") => "light".to_string(),
        _ => "dark".to_string(),
    }
}

fn default_ui_state() -> StoredUiStateV1 {
    StoredUiStateV1 {
        version: UI_STATE_SCHEMA_VERSION,
        last_input_dir: None,
        theme: None,
    }
}

fn read_stored_ui_state(path: &Path) -> StoredUiStateV1 {
    if !path.exists() {
        return default_ui_state();
    }
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(_) => return default_ui_state(),
    };
    match serde_json::from_str::<StoredUiStateV1>(&text) {
        Ok(state) if state.version == UI_STATE_SCHEMA_VERSION => state,
        _ => default_ui_state(),
    }
}

fn write_ui_state(path: &Path, state: &StoredUiStateV1) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let text = serde_json::to_string_pretty(state)
        .map_err(|e| format!("could not serialize ui state: {e}"))?;
    std::fs::write(path, text)
        .map_err(|e| format!("could not write ui state file {}: {e}", path.display()))
}

fn load_ui_state_from_path(path: &Path) -> UiStateResponse {
    let state = read_stored_ui_state(path);
    UiStateResponse {
        last_input_dir: state
            .last_input_dir
            .filter(|value| !value.trim().is_empty()),
        theme: normalize_theme(state.theme),
    }
}

fn save_last_input_dir_to_path(path: &Path, input_dir: &str) -> Result<(), String> {
    let trimmed = input_dir.trim();
    if trimmed.is_empty() {
        return Ok(());
    }
    let mut state = read_stored_ui_state(path);
    state.last_input_dir = Some(trimmed.to_string());
    write_ui_state(path, &state)
}

fn save_theme_to_path(path: &Path, theme: &str) -> Result<(), String> {
    let mut state = read_stored_ui_state(path);
    state.theme = Some(normalize_theme(Some(theme.to_string())));
    write_ui_state(path, &state)
}

fn load_settings_from_path(path: &Path) -> Result<Config, String> {
    if !path.exists() {
        return Ok(default_config());
    }
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("could not read settings file {}: {e}", path.display()))?;
    parse_stored_settings(&text)
}

fn save_settings_to_path(path: &Path, settings: &Config) -> Result<(), String> {
    similar_textures_core::config::validate(settings)
        .map_err(|e| format!("settings validation failed: {e}"))?;
    let text = serialize_stored_settings(settings)?;
    std::fs::write(path, text)
        .map_err(|e| format!("could not write settings file {}: {e}", path.display()))
}

fn emit_log(app: &AppHandle, scan_id: u64, line: impl Into<String>, level: &str) {
    let payload = ScanLogEventPayload {
        scan_id,
        line: line.into(),
        level: level.to_string(),
    };
    let _ = app.emit("scan-log", payload);
}

fn emit_progress(
    app: &AppHandle,
    scan_id: u64,
    phase: &str,
    processed: Option<usize>,
    total: Option<usize>,
    vertices: Option<usize>,
    groups: Option<usize>,
) {
    let payload = ScanProgressEventPayload {
        scan_id,
        phase: phase.to_string(),
        processed,
        total,
        vertices,
        groups,
    };
    let _ = app.emit("scan-progress", payload);
}

fn emit_finished(app: &AppHandle, scan_id: u64, status: &str, message: Option<String>) {
    let payload = ScanFinishedEventPayload {
        scan_id,
        status: status.to_string(),
        message,
    };
    let _ = app.emit("scan-finished", payload);
}

struct EventCallbacks {
    app: AppHandle,
    scan_id: u64,
    cancel_flag: Arc<AtomicBool>,
}

impl ScanCallbacks for EventCallbacks {
    fn is_cancelled(&self) -> bool {
        self.cancel_flag.load(Ordering::Relaxed)
    }

    fn on_event(&self, event: ScanEvent) {
        match event {
            ScanEvent::ScanDone { scanned_paths } => {
                emit_log(
                    &self.app,
                    self.scan_id,
                    format!("scan done: {scanned_paths} paths"),
                    "info",
                );
                emit_progress(
                    &self.app,
                    self.scan_id,
                    "scan_done",
                    None,
                    None,
                    Some(scanned_paths),
                    None,
                );
            }
            ScanEvent::ProcessedImage { processed, total } => {
                emit_progress(
                    &self.app,
                    self.scan_id,
                    "ingest_progress",
                    Some(processed),
                    Some(total),
                    None,
                    None,
                );
            }
            ScanEvent::IngestDone {
                vertices,
                scanned_paths,
                decode_ok,
                prepare_ok,
                hash_skip,
                threads,
            } => {
                emit_log(
                    &self.app,
                    self.scan_id,
                    format!(
                        "ingest done: vertices={vertices} scanned={scanned_paths} decode_ok={decode_ok} prepare_ok={prepare_ok} hash_skip={hash_skip} threads={threads}"
                    ),
                    "info",
                );
                emit_progress(
                    &self.app,
                    self.scan_id,
                    "ingest_done",
                    None,
                    None,
                    Some(vertices),
                    None,
                );
            }
            ScanEvent::HashGroupsDone {
                hash_edges,
                duplicate_vertices,
            } => {
                emit_log(
                    &self.app,
                    self.scan_id,
                    format!(
                        "hash groups: hash_edges={hash_edges} duplicate_vertices={duplicate_vertices}"
                    ),
                    "info",
                );
            }
            ScanEvent::PairwiseStart {
                vertices,
                total_pairs,
                hash_pairs,
                threads,
            } => {
                emit_log(
                    &self.app,
                    self.scan_id,
                    format!(
                        "pairwise start: vertices={vertices} total_pairs={total_pairs} hash_pairs={hash_pairs} threads={threads}"
                    ),
                    "info",
                );
                emit_progress(
                    &self.app,
                    self.scan_id,
                    "pairwise_start",
                    None,
                    None,
                    Some(vertices),
                    None,
                );
            }
            ScanEvent::PairwiseDone {
                hash_edges,
                composite_edges,
                score_cache,
            } => {
                emit_log(
                    &self.app,
                    self.scan_id,
                    format!(
                        "pairwise done: hash_edges={hash_edges} composite_edges={composite_edges} score_cache={score_cache}"
                    ),
                    "info",
                );
            }
            ScanEvent::ClusteringDone { groups } => {
                emit_progress(
                    &self.app,
                    self.scan_id,
                    "clustering_done",
                    None,
                    None,
                    None,
                    Some(groups),
                );
                emit_log(
                    &self.app,
                    self.scan_id,
                    format!("clustering done: groups={groups}"),
                    "info",
                );
            }
            ScanEvent::WritingDone => {
                emit_log(&self.app, self.scan_id, "result prepared in memory", "info");
            }
            ScanEvent::Decoded { .. } | ScanEvent::Prepared { .. } => {}
            ScanEvent::DecodeFailed { path, message } => {
                emit_log(
                    &self.app,
                    self.scan_id,
                    format!("decode failed {}: {message}", path.display()),
                    "warn",
                );
            }
            ScanEvent::PrepareFailed { path, message } => {
                emit_log(
                    &self.app,
                    self.scan_id,
                    format!("prepare failed {}: {message}", path.display()),
                    "warn",
                );
            }
            ScanEvent::FeaturesFailed { path, message } => {
                emit_log(
                    &self.app,
                    self.scan_id,
                    format!("features failed {}: {message}", path.display()),
                    "warn",
                );
            }
            ScanEvent::HashSkipped { path, message } => {
                emit_log(
                    &self.app,
                    self.scan_id,
                    format!("hash skip {}: {message}", path.display()),
                    "warn",
                );
            }
        }
    }
}

#[tauri::command]
fn load_ui_state(app: AppHandle) -> Result<UiStateResponse, String> {
    let path = ui_state_path(&app)?;
    Ok(load_ui_state_from_path(&path))
}

#[tauri::command]
fn save_last_input_dir(app: AppHandle, input_dir: String) -> Result<(), String> {
    let path = ui_state_path(&app)?;
    save_last_input_dir_to_path(&path, &input_dir)
}

#[tauri::command]
fn save_app_theme(app: AppHandle, theme: String) -> Result<(), String> {
    let normalized = normalize_theme(Some(theme));
    if normalized != "dark" && normalized != "light" {
        return Err("theme must be \"dark\" or \"light\"".to_string());
    }
    let path = ui_state_path(&app)?;
    save_theme_to_path(&path, &normalized)
}

#[tauri::command]
fn count_images(request: StartScanRequest) -> Result<CountImagesResponse, String> {
    let input_dir = request.input_dir.trim();
    if input_dir.is_empty() {
        return Err("input_dir is required".to_string());
    }
    let input = PathBuf::from(input_dir);
    if !input.exists() {
        return Err(format!("input path does not exist: {}", input.display()));
    }
    let meta = std::fs::metadata(&input).map_err(|e| e.to_string())?;
    if !meta.is_dir() {
        return Err(format!("input path is not a directory: {}", input.display()));
    }
    let image_count = similar_textures_core::scanner::scan_images(&input).len();
    Ok(CountImagesResponse { image_count })
}

#[tauri::command]
fn load_settings(app: AppHandle) -> Result<Config, String> {
    let path = settings_path(&app)?;
    load_settings_from_path(&path)
}

#[tauri::command]
fn save_settings(app: AppHandle, settings: Config) -> Result<(), String> {
    let path = settings_path(&app)?;
    save_settings_to_path(&path, &settings)
}

#[tauri::command]
fn scan_status(slots: State<AppSlots>) -> Result<ScanStatusResponse, String> {
    let status = *slots.status.lock().map_err(|_| "lock poisoned")?;
    let active = slots.active_scan_id.load(Ordering::Relaxed);
    Ok(ScanStatusResponse {
        state: status.as_str().to_string(),
        active_scan_id: (active != 0).then_some(active),
    })
}

#[tauri::command]
fn get_last_result(slots: State<AppSlots>) -> Result<LastResultResponse, String> {
    let result = slots
        .last_result
        .lock()
        .map_err(|_| "lock poisoned")?
        .clone();
    Ok(LastResultResponse {
        has_result: result.is_some(),
        result,
    })
}

#[tauri::command]
fn cancel_scan(slots: State<AppSlots>) -> Result<(), String> {
    let status = *slots.status.lock().map_err(|_| "lock poisoned")?;
    if status != ScanStatusValue::Running {
        return Ok(());
    }
    slots.cancel_flag.store(true, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
fn start_scan(
    app: AppHandle,
    slots: State<AppSlots>,
    request: StartScanRequest,
) -> Result<StartScanResponse, String> {
    let mut status_guard = slots.status.lock().map_err(|_| "lock poisoned")?;
    if *status_guard == ScanStatusValue::Running {
        return Err("scan already running".to_string());
    }
    *status_guard = ScanStatusValue::Running;
    drop(status_guard);

    slots.cancel_flag.store(false, Ordering::Relaxed);

    let scan_id = slots.scan_sequence.fetch_add(1, Ordering::Relaxed) + 1;
    slots.active_scan_id.store(scan_id, Ordering::Relaxed);

    let input_dir = request.input_dir.trim();
    let input = PathBuf::from(input_dir);
    let threads = request.threads.unwrap_or_else(default_threads).max(1);
    if let Ok(ui_path) = ui_state_path(&app) {
        let _ = save_last_input_dir_to_path(&ui_path, input_dir);
    }
    let cfg = load_settings(app.clone())?;
    emit_log(
        &app,
        scan_id,
        format!("scan start: input={} threads={threads}", input.display()),
        "info",
    );

    let app_handle = app.clone();
    let cancel_flag = slots.cancel_flag.clone();

    thread::spawn(move || {
        let callbacks = EventCallbacks {
            app: app_handle.clone(),
            scan_id,
            cancel_flag: cancel_flag.clone(),
        };
        let core_request = ScanRequest { input, threads };
        let run_result = run_scan(&cfg, &core_request, &callbacks);

        let app_slots = app_handle.state::<AppSlots>();
        let mut status = app_slots.status.lock().expect("status lock");
        match run_result {
            Ok(ScanOutcome {
                status: ScanStatus::Completed,
                result,
                ..
            }) => {
                *app_slots.last_result.lock().expect("result lock") = Some(result);
                *status = ScanStatusValue::Completed;
                emit_finished(&app_handle, scan_id, "completed", None);
            }
            Ok(ScanOutcome {
                status: ScanStatus::Cancelled,
                ..
            }) => {
                *status = ScanStatusValue::Cancelled;
                emit_finished(&app_handle, scan_id, "cancelled", None);
            }
            Err(err) => {
                *status = ScanStatusValue::Failed;
                emit_log(&app_handle, scan_id, format!("scan failed: {err}"), "error");
                emit_finished(&app_handle, scan_id, "failed", Some(err.to_string()));
            }
        }
        app_slots.active_scan_id.store(0, Ordering::Relaxed);
    });

    Ok(StartScanResponse { scan_id })
}

#[tauri::command]
fn export_last_result_json(
    slots: State<AppSlots>,
    request: ExportRequest,
) -> Result<ExportResponse, String> {
    let path = request
        .target_path
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| "target_path is required".to_string())?;
    let output_path = PathBuf::from(path);

    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }

    let result = slots
        .last_result
        .lock()
        .map_err(|_| "lock poisoned")?
        .clone()
        .ok_or_else(|| "no scan result in memory".to_string())?;

    write_result_json(Path::new(&output_path), &result).map_err(|e| e.to_string())?;
    Ok(ExportResponse {
        path: output_path.to_string_lossy().to_string(),
    })
}

#[tauri::command]
fn export_last_result_csv(
    slots: State<AppSlots>,
    request: ExportRequest,
) -> Result<ExportResponse, String> {
    let path = request
        .target_path
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| "target_path is required".to_string())?;
    let output_path = PathBuf::from(path);

    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }

    let result = slots
        .last_result
        .lock()
        .map_err(|_| "lock poisoned")?
        .clone()
        .ok_or_else(|| "no scan result in memory".to_string())?;

    write_result_csv(Path::new(&output_path), &result).map_err(|e| e.to_string())?;
    Ok(ExportResponse {
        path: output_path.to_string_lossy().to_string(),
    })
}

fn preview_cache_key(source: &Path, max_px: u32) -> String {
    let mut hasher = DefaultHasher::new();
    source.to_string_lossy().hash(&mut hasher);
    if let Ok(meta) = std::fs::metadata(source) {
        meta.len().hash(&mut hasher);
        if let Ok(modified) = meta.modified() {
            modified.hash(&mut hasher);
        }
    }
    max_px.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn preview_cache_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_cache_dir().map_err(|e| e.to_string())?;
    Ok(dir.join("previews"))
}

#[tauri::command]
async fn get_image_preview(
    app: AppHandle,
    request: ImagePreviewRequest,
) -> Result<ImagePreviewResponse, String> {
    let source = PathBuf::from(request.path.trim());
    if !source.is_file() {
        return Err(format!("image not found: {}", source.display()));
    }
    let max_px = request.max_px.unwrap_or(DEFAULT_PREVIEW_MAX_PX).max(1);
    let cache_dir = preview_cache_dir(&app)?;
    let cache_path = cache_dir.join(format!("{}.jpg", preview_cache_key(&source, max_px)));
    if !cache_path.exists() {
        let source_for_preview = source.clone();
        let cache_path_for_preview = cache_path.clone();
        tauri::async_runtime::spawn_blocking(move || {
            if cache_path_for_preview.exists() {
                return Ok(());
            }
            ImageData::write_preview_jpeg(&source_for_preview, &cache_path_for_preview, max_px)
        })
        .await
        .map_err(|e| format!("preview task failed: {e}"))??;
    }
    Ok(ImagePreviewResponse {
        path: cache_path.to_string_lossy().to_string(),
    })
}

#[tauri::command]
fn load_result_json(
    slots: State<AppSlots>,
    request: LoadResultRequest,
) -> Result<LoadResultResponse, String> {
    let source_path = request.source_path.trim();
    if source_path.is_empty() {
        return Err("source_path is required".to_string());
    }
    let path = PathBuf::from(source_path);
    let result = read_result_json(&path)?;
    let group_count = result.groups.len();
    *slots
        .last_result
        .lock()
        .map_err(|_| "lock poisoned")? = Some(result);
    Ok(LoadResultResponse { group_count })
}

fn default_threads() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

fn default_config() -> Config {
    serde_json::from_str(
        r#"{
            "enable_phash": true,
            "enable_ssim": true,
            "enable_histogram": true,
            "enable_alpha_crop": false,
            "enable_rotations": false,
            "enable_flip": false,
            "threshold": 0.85,
            "weights": { "phash": 0.35, "ssim": 0.45, "histogram": 0.2 },
            "hash_algorithm": "sha256",
            "phash_max_distance": 10,
            "ssim_threshold": 0.9,
            "resize_size": 256,
            "hist_bins": 512,
            "hist_method": "correlation",
            "alpha_threshold": 0.05,
            "hide_single_image_groups": true
        }"#,
    )
    .expect("valid embedded default config")
}

fn parse_stored_settings(text: &str) -> Result<Config, String> {
    let parsed: StoredSettingsFile =
        serde_json::from_str(text).map_err(|e| format!("settings file is invalid JSON: {e}"))?;
    let cfg = match parsed {
        StoredSettingsFile::V1(v1) => {
            if v1.version != SETTINGS_SCHEMA_VERSION {
                return Err(format!(
                    "settings schema version {} is not supported (expected {})",
                    v1.version, SETTINGS_SCHEMA_VERSION
                ));
            }
            v1.config
        }
        StoredSettingsFile::Legacy(cfg) => cfg,
    };
    similar_textures_core::config::validate(&cfg)
        .map_err(|e| format!("settings file is invalid: {e}"))?;
    Ok(cfg)
}

fn serialize_stored_settings(cfg: &Config) -> Result<String, String> {
    let wrapped = StoredSettingsV1 {
        version: SETTINGS_SCHEMA_VERSION,
        config: cfg.clone(),
    };
    serde_json::to_string_pretty(&wrapped)
        .map_err(|e| format!("could not serialize settings payload: {e}"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AppSlots::default())
        .invoke_handler(tauri::generate_handler![
            load_ui_state,
            save_last_input_dir,
            save_app_theme,
            count_images,
            load_settings,
            save_settings,
            scan_status,
            get_last_result,
            start_scan,
            cancel_scan,
            export_last_result_json,
            export_last_result_csv,
            get_image_preview,
            load_result_json,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn default_config_is_valid() {
        let cfg = default_config();
        similar_textures_core::config::validate(&cfg).expect("default config must validate");
    }

    #[test]
    fn parse_legacy_settings_shape() {
        let cfg = default_config();
        let text = serde_json::to_string(&cfg).expect("serialize");
        let loaded = parse_stored_settings(&text).expect("parse");
        assert_eq!(loaded.hash_algorithm, "sha256");
        assert!((loaded.threshold - 0.85).abs() < 1e-9);
    }

    #[test]
    fn serialize_and_parse_versioned_settings_shape() {
        let cfg = default_config();
        let text = serialize_stored_settings(&cfg).expect("serialize wrapped");
        let loaded = parse_stored_settings(&text).expect("parse wrapped");
        assert_eq!(loaded.hash_algorithm, cfg.hash_algorithm);
        assert_eq!(loaded.phash_max_distance, cfg.phash_max_distance);
    }

    #[test]
    fn reject_unsupported_settings_version() {
        let cfg = default_config();
        let payload = serde_json::json!({
            "version": SETTINGS_SCHEMA_VERSION + 1,
            "config": cfg
        });
        let text = serde_json::to_string(&payload).expect("json");
        let err = parse_stored_settings(&text).expect_err("must reject");
        assert!(err.contains("not supported"));
    }

    #[test]
    fn reject_invalid_config_payload() {
        let payload = serde_json::json!({
            "version": SETTINGS_SCHEMA_VERSION,
            "config": {
                "enable_phash": false,
                "enable_ssim": false,
                "enable_histogram": false,
                "threshold": 0.85,
                "weights": { "phash": 0.35, "ssim": 0.45, "histogram": 0.2 }
            }
        });
        let text = serde_json::to_string(&payload).expect("json");
        let err = parse_stored_settings(&text).expect_err("must fail validation");
        assert!(err.contains("at least one of enable_phash"));
    }

    #[test]
    fn save_and_load_settings_path_roundtrip() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("settings.json");
        let mut cfg = default_config();
        cfg.threshold = 0.77;
        save_settings_to_path(&path, &cfg).expect("save");
        let loaded = load_settings_from_path(&path).expect("load");
        assert!((loaded.threshold - 0.77).abs() < 1e-9);
    }

    #[test]
    fn save_and_load_ui_state_roundtrip() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("ui-state.json");
        save_last_input_dir_to_path(&path, "/tmp/textures").expect("save");
        let loaded = load_ui_state_from_path(&path);
        assert_eq!(loaded.last_input_dir.as_deref(), Some("/tmp/textures"));
        assert_eq!(loaded.theme, "dark");
    }

    #[test]
    fn save_and_load_ui_state_theme_roundtrip() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("ui-state.json");
        save_theme_to_path(&path, "light").expect("save theme");
        save_last_input_dir_to_path(&path, "/tmp/textures").expect("save dir");
        let loaded = load_ui_state_from_path(&path);
        assert_eq!(loaded.theme, "light");
        assert_eq!(loaded.last_input_dir.as_deref(), Some("/tmp/textures"));
    }

    #[test]
    fn read_result_json_roundtrip() {
        let dir = tempdir().expect("tempdir");
        let json_path = dir.path().join("result.json");
        let sample = similar_textures_core::ScanResult {
            groups: vec![similar_textures_core::ScanGroup {
                id: 1,
                score: Some(0.9),
                images: vec!["/tmp/a.png".to_string()],
                reason_kind: similar_textures_core::GroupReasonKind::Singleton,
                reasons: vec![],
            }],
        };
        similar_textures_core::write_result_json(&json_path, &sample).expect("write");
        let loaded = similar_textures_core::output::read_result_json(&json_path).expect("read");
        assert_eq!(loaded.groups.len(), 1);
        assert_eq!(loaded.groups[0].id, 1);
    }

    #[test]
    fn export_csv_contains_expected_columns() {
        let dir = tempdir().expect("tempdir");
        let csv_path = dir.path().join("result.csv");
        let sample = similar_textures_core::ScanResult {
            groups: vec![similar_textures_core::ScanGroup {
                id: 3,
                score: Some(0.75),
                images: vec!["/tmp/a.png".to_string(), "/tmp/b.png".to_string()],
                reason_kind: similar_textures_core::GroupReasonKind::Composite,
                reasons: vec![],
            }],
        };
        similar_textures_core::write_result_csv(&csv_path, &sample).expect("write csv");
        let text = std::fs::read_to_string(&csv_path).expect("read csv");
        assert!(text.starts_with("group_id,group_name,score,reason_kind,image_path\n"));
        assert!(text.contains("3,Group #3,0.75,composite,/tmp/a.png"));
        assert!(text.contains("/tmp/b.png"));
    }

    #[test]
    fn saved_settings_file_is_versioned_payload() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("settings.json");
        let cfg = default_config();
        save_settings_to_path(&path, &cfg).expect("save");
        let text = std::fs::read_to_string(&path).expect("read");
        let value: serde_json::Value = serde_json::from_str(&text).expect("json");
        assert_eq!(value.get("version").and_then(|v| v.as_u64()), Some(1));
        assert!(value.get("config").is_some());
    }
}

