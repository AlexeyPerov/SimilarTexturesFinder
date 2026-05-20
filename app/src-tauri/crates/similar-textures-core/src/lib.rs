pub mod clustering;
pub mod config;
pub mod features;
pub mod file_hash;
pub mod group_score;
pub mod image_loader;
pub mod output;
pub mod pairwise;
pub mod scanner;
pub mod similarity;
pub mod vertex;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use features::extract::extract_features;
use image_loader::ImageData;
use serde::{Deserialize, Serialize};
use vertex::Vertex;

pub const EXIT_CANCELLED: u8 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScanStatus {
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanRequest {
    pub input: PathBuf,
    pub threads: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanGroup {
    pub id: u32,
    pub score: Option<f64>,
    pub images: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub groups: Vec<ScanGroup>,
}

#[derive(Debug, Clone)]
pub struct ScanMetrics {
    pub scanned_paths: usize,
    pub vertices: usize,
    pub hash_edges: usize,
    pub composite_edges: usize,
}

#[derive(Debug, Clone)]
pub struct ScanOutcome {
    pub status: ScanStatus,
    pub result: ScanResult,
    pub metrics: ScanMetrics,
}

#[derive(Debug)]
pub enum ScanError {
    InputNotFound(PathBuf),
    InputNotDirectory(PathBuf),
    InputNotAccessible { path: PathBuf, error: std::io::Error },
    UnknownHashAlgorithm(String),
    CanonicalizePath { path: PathBuf, error: std::io::Error },
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanError::InputNotFound(path) => {
                write!(f, "input does not exist: {}", path.display())
            }
            ScanError::InputNotDirectory(path) => {
                write!(f, "input must be a directory: {}", path.display())
            }
            ScanError::InputNotAccessible { path, error } => {
                write!(f, "input not accessible: {} ({error})", path.display())
            }
            ScanError::UnknownHashAlgorithm(raw) => {
                write!(
                    f,
                    "unknown hash_algorithm {raw:?} (supported: md5, sha1, sha256)"
                )
            }
            ScanError::CanonicalizePath { path, error } => {
                write!(f, "could not canonicalize {}: {error}", path.display())
            }
        }
    }
}

impl std::error::Error for ScanError {}

#[derive(Debug, Clone)]
pub enum ScanEvent {
    ScanDone {
        scanned_paths: usize,
    },
    ProcessedImage {
        processed: usize,
        total: usize,
    },
    IngestDone {
        vertices: usize,
        scanned_paths: usize,
        decode_ok: u64,
        prepare_ok: u64,
        hash_skip: u64,
        threads: usize,
    },
    HashGroupsDone {
        hash_edges: usize,
        duplicate_vertices: usize,
    },
    PairwiseStart {
        vertices: usize,
        total_pairs: u128,
        hash_pairs: u128,
        threads: usize,
    },
    PairwiseDone {
        hash_edges: usize,
        composite_edges: usize,
        score_cache: usize,
    },
    ClusteringDone {
        groups: usize,
    },
    WritingDone,
    Decoded {
        path: PathBuf,
    },
    DecodeFailed {
        path: PathBuf,
        message: String,
    },
    Prepared {
        path: PathBuf,
    },
    PrepareFailed {
        path: PathBuf,
        message: String,
    },
    FeaturesFailed {
        path: PathBuf,
        message: String,
    },
    HashSkipped {
        path: PathBuf,
        message: String,
    },
}

pub trait ScanCallbacks: Send + Sync {
    fn is_cancelled(&self) -> bool {
        false
    }

    fn on_event(&self, _event: ScanEvent) {}
}

pub struct NoopCallbacks;

impl ScanCallbacks for NoopCallbacks {}

pub fn run_scan(
    cfg: &config::Config,
    request: &ScanRequest,
    callbacks: &dyn ScanCallbacks,
) -> Result<ScanOutcome, ScanError> {
    let input = request.input.as_path();
    if !input.exists() {
        return Err(ScanError::InputNotFound(input.to_path_buf()));
    }
    let meta = std::fs::metadata(input).map_err(|error| ScanError::InputNotAccessible {
        path: input.to_path_buf(),
        error,
    })?;
    if !meta.is_dir() {
        return Err(ScanError::InputNotDirectory(input.to_path_buf()));
    }

    let hash_algo = cfg
        .hash_algorithm()
        .map_err(ScanError::UnknownHashAlgorithm)?;

    let paths = scanner::scan_images(input);
    let n_scanned = paths.len();
    callbacks.on_event(ScanEvent::ScanDone {
        scanned_paths: n_scanned,
    });

    if callbacks.is_cancelled() {
        return Ok(ScanOutcome {
            status: ScanStatus::Cancelled,
            result: ScanResult { groups: Vec::new() },
            metrics: ScanMetrics {
                scanned_paths: n_scanned,
                vertices: 0,
                hash_edges: 0,
                composite_edges: 0,
            },
        });
    }

    let mut hash_cache = file_hash::FileHashCache::new(hash_algo);
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut n_hash_fail = 0u64;
    let mut n_decode_ok = 0u64;
    let mut n_prepare_ok = 0u64;
    const INGEST_PROGRESS_EVERY: usize = 30;

    for (i, path) in paths.into_iter().enumerate() {
        if callbacks.is_cancelled() {
            return Ok(ScanOutcome {
                status: ScanStatus::Cancelled,
                result: ScanResult { groups: Vec::new() },
                metrics: ScanMetrics {
                    scanned_paths: n_scanned,
                    vertices: vertices.len(),
                    hash_edges: 0,
                    composite_edges: 0,
                },
            });
        }

        let ingest_ix = i + 1;
        match hash_cache.digest_for_path(&path) {
            Ok(digest) => {
                let features = match ImageData::decode_path(&path, cfg.max_decode_dimension_px) {
                    Ok(ref img) => {
                        n_decode_ok += 1;
                        callbacks.on_event(ScanEvent::Decoded { path: path.clone() });
                        match similarity::prepare_vertex(img, cfg) {
                            Ok(prepared) => {
                                n_prepare_ok += 1;
                                callbacks.on_event(ScanEvent::Prepared { path: path.clone() });
                                match extract_features(&prepared, cfg) {
                                    Ok(f) => Some(f),
                                    Err(message) => {
                                        callbacks.on_event(ScanEvent::FeaturesFailed {
                                            path: path.clone(),
                                            message,
                                        });
                                        None
                                    }
                                }
                            }
                            Err(message) => {
                                callbacks.on_event(ScanEvent::PrepareFailed {
                                    path: path.clone(),
                                    message,
                                });
                                None
                            }
                        }
                    }
                    Err(message) => {
                        callbacks.on_event(ScanEvent::DecodeFailed {
                            path: path.clone(),
                            message,
                        });
                        None
                    }
                };
                vertices.push(Vertex {
                    path,
                    digest,
                    features,
                });
            }
            Err(error) => {
                n_hash_fail += 1;
                callbacks.on_event(ScanEvent::HashSkipped {
                    path: path.clone(),
                    message: error.to_string(),
                });
            }
        }

        if n_scanned > 0 && INGEST_PROGRESS_EVERY > 0 {
            let at_interval = ingest_ix % INGEST_PROGRESS_EVERY == 0;
            let at_end = ingest_ix == n_scanned;
            if at_interval || at_end {
                callbacks.on_event(ScanEvent::ProcessedImage {
                    processed: ingest_ix,
                    total: n_scanned,
                });
            }
        }
    }

    let threads = request.threads.max(1);
    callbacks.on_event(ScanEvent::IngestDone {
        vertices: vertices.len(),
        scanned_paths: n_scanned,
        decode_ok: n_decode_ok,
        prepare_ok: n_prepare_ok,
        hash_skip: n_hash_fail,
        threads,
    });

    if callbacks.is_cancelled() {
        return Ok(ScanOutcome {
            status: ScanStatus::Cancelled,
            result: ScanResult { groups: Vec::new() },
            metrics: ScanMetrics {
                scanned_paths: n_scanned,
                vertices: vertices.len(),
                hash_edges: 0,
                composite_edges: 0,
            },
        });
    }

    let n = vertices.len();
    let (hash_edges, digest_rep) = build_hash_groups(&vertices);
    if !hash_edges.is_empty() {
        let duplicate_vertices = digest_rep
            .iter()
            .enumerate()
            .filter(|(i, r)| **r != *i)
            .count();
        callbacks.on_event(ScanEvent::HashGroupsDone {
            hash_edges: hash_edges.len(),
            duplicate_vertices,
        });
    }

    let n_pairs: u128 = if n >= 2 {
        (n as u128) * ((n - 1) as u128) / 2
    } else {
        0
    };
    let n_hash_pairs = hash_edges.len() as u128;
    callbacks.on_event(ScanEvent::PairwiseStart {
        vertices: n,
        total_pairs: n_pairs,
        hash_pairs: n_hash_pairs,
        threads,
    });

    let stats = pairwise::pairwise_compare(cfg, &vertices, threads, &digest_rep, callbacks);

    if callbacks.is_cancelled() {
        return Ok(ScanOutcome {
            status: ScanStatus::Cancelled,
            result: ScanResult { groups: Vec::new() },
            metrics: ScanMetrics {
                scanned_paths: n_scanned,
                vertices: n,
                hash_edges: hash_edges.len(),
                composite_edges: stats.composite_edges.len(),
            },
        });
    }

    callbacks.on_event(ScanEvent::PairwiseDone {
        hash_edges: hash_edges.len(),
        composite_edges: stats.composite_edges.len(),
        score_cache: stats.score_cache.len(),
    });

    let group_indices = clustering::build_groups(n, &hash_edges, &stats.composite_edges, &vertices);
    callbacks.on_event(ScanEvent::ClusteringDone {
        groups: group_indices.len(),
    });
    let group_scores = group_score::group_scores(&group_indices, &vertices, &stats.score_cache);

    let mut groups_out = Vec::with_capacity(group_indices.len());
    for (gi, members) in group_indices.iter().enumerate() {
        let id = (gi + 1) as u32;
        let mut images = Vec::with_capacity(members.len());
        for &idx in members {
            let image = canonical_path_string(&vertices[idx].path).map_err(|error| {
                ScanError::CanonicalizePath {
                    path: vertices[idx].path.clone(),
                    error,
                }
            })?;
            images.push(image);
        }
        groups_out.push(ScanGroup {
            id,
            score: group_scores[gi],
            images,
        });
    }

    callbacks.on_event(ScanEvent::WritingDone);
    Ok(ScanOutcome {
        status: ScanStatus::Completed,
        result: ScanResult { groups: groups_out },
        metrics: ScanMetrics {
            scanned_paths: n_scanned,
            vertices: n,
            hash_edges: hash_edges.len(),
            composite_edges: stats.composite_edges.len(),
        },
    })
}

pub fn write_result_json(path: &Path, result: &ScanResult) -> std::io::Result<()> {
    let groups = result
        .groups
        .iter()
        .map(|g| output::GroupRecord {
            id: g.id,
            score: g.score,
            images: g.images.clone(),
        })
        .collect();
    let doc = output::ResultJson { groups };
    output::write_result_json(path, &doc)
}

fn build_hash_groups(vertices: &[Vertex]) -> (Vec<(usize, usize)>, Vec<usize>) {
    let mut groups: HashMap<Vec<u8>, Vec<usize>> = HashMap::new();
    for (i, v) in vertices.iter().enumerate() {
        groups.entry(v.digest.clone()).or_default().push(i);
    }

    let mut hash_edges = Vec::new();
    let mut rep: Vec<usize> = (0..vertices.len()).collect();

    for members in groups.values() {
        if members.len() >= 2 {
            let first = members[0];
            for &m in members {
                rep[m] = first;
            }
            for ai in 0..members.len() {
                for bi in (ai + 1)..members.len() {
                    hash_edges.push((members[ai], members[bi]));
                }
            }
        }
    }

    (hash_edges, rep)
}

fn canonical_path_string(p: &Path) -> Result<String, std::io::Error> {
    let c = std::fs::canonicalize(p)?;
    Ok(c.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn default_cfg() -> config::Config {
        let json = r#"{
            "enable_phash": true,
            "enable_ssim": true,
            "enable_histogram": true,
            "enable_alpha_crop": false,
            "enable_rotations": false,
            "enable_flip": false,
            "threshold": 0.85,
            "weights": { "phash": 0.35, "ssim": 0.45, "histogram": 0.2 }
        }"#;
        serde_json::from_str(json).expect("valid cfg")
    }

    #[test]
    fn groups_identical_images_together() {
        let dir = tempdir().expect("tempdir");
        let input_dir = dir.path().join("input");
        fs::create_dir_all(&input_dir).expect("mkdir");
        let a = input_dir.join("a.png");
        let b = input_dir.join("b.png");
        let c = input_dir.join("c.png");

        let data = tiny_png_rgba(8, 8, [255, 0, 0, 255]);
        fs::write(&a, &data).expect("write a");
        fs::write(&b, &data).expect("write b");
        let different = tiny_png_rgba(8, 8, [0, 255, 0, 255]);
        fs::write(&c, &different).expect("write c");

        let cfg = default_cfg();
        let req = ScanRequest {
            input: input_dir,
            threads: 2,
        };
        let outcome = run_scan(&cfg, &req, &NoopCallbacks).expect("scan");
        assert_eq!(outcome.status, ScanStatus::Completed);
        assert_eq!(outcome.metrics.scanned_paths, 3);
        assert!(outcome.metrics.hash_edges >= 1);
        assert!(
            outcome.result.groups.iter().any(|g| g.images.len() >= 2),
            "at least one non-singleton group expected"
        );
    }

    #[test]
    fn can_export_json_from_in_memory_result() {
        let result = ScanResult {
            groups: vec![ScanGroup {
                id: 1,
                score: Some(1.0),
                images: vec!["/tmp/a.png".to_string(), "/tmp/b.png".to_string()],
            }],
        };
        let dir = tempdir().expect("tempdir");
        let out = dir.path().join("result.json");
        write_result_json(&out, &result).expect("write");
        let text = fs::read_to_string(out).expect("read");
        assert!(text.contains("\"groups\""));
        assert!(text.contains("\"images\""));
    }

    struct CancelImmediately;

    impl ScanCallbacks for CancelImmediately {
        fn is_cancelled(&self) -> bool {
            true
        }
    }

    #[test]
    fn immediate_cancellation_returns_cancelled_status() {
        let dir = tempdir().expect("tempdir");
        let input_dir = dir.path().join("input");
        fs::create_dir_all(&input_dir).expect("mkdir");
        let cfg = default_cfg();
        let req = ScanRequest {
            input: input_dir,
            threads: 2,
        };
        let outcome = run_scan(&cfg, &req, &CancelImmediately).expect("scan");
        assert_eq!(outcome.status, ScanStatus::Cancelled);
    }

    fn tiny_png_rgba(w: u32, h: u32, rgba: [u8; 4]) -> Vec<u8> {
        let mut img = image::RgbaImage::new(w, h);
        for p in img.pixels_mut() {
            *p = image::Rgba(rgba);
        }
        let dyn_img = image::DynamicImage::ImageRgba8(img);
        let mut buf = Vec::new();
        {
            let mut cursor = std::io::Cursor::new(&mut buf);
            dyn_img
                .write_to(&mut cursor, image::ImageFormat::Png)
                .expect("encode png");
        }
        buf
    }
}
