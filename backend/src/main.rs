mod clustering;
mod config;
mod features;
mod file_hash;
mod group_score;
mod image_loader;
mod output;
mod pairwise;
mod scanner;
mod similarity;
mod vertex;

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use clap::Parser;

use crate::features::extract::extract_features;
use crate::image_loader::ImageData;
use crate::similarity::prepare_vertex;
use crate::vertex::Vertex;

#[derive(Parser, Debug)]
#[command(name = "texture_tool", version, about = "Similar texture finder (MVP backend)")]
struct Args {
    /// Root folder to scan (recursive).
    #[arg(long)]
    input: PathBuf,

    /// Path for `result.json`.
    #[arg(long)]
    output: PathBuf,

    /// Path to `config.json`.
    #[arg(long)]
    config: PathBuf,

    /// Worker threads for pairwise comparison (`rayon` pool size).
    #[arg(long)]
    threads: Option<usize>,
}

fn main() -> ExitCode {
    if let Err(code) = run() {
        return code;
    }
    ExitCode::SUCCESS
}

fn run() -> Result<(), ExitCode> {
    let args = Args::parse();

    let input = args.input.as_path();
    if !input.exists() {
        eprintln!("error: --input does not exist: {}", input.display());
        return Err(ExitCode::from(1));
    }
    let meta = match std::fs::metadata(input) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("error: --input not accessible: {} ({e})", input.display());
            return Err(ExitCode::from(1));
        }
    };
    if !meta.is_dir() {
        eprintln!("error: --input must be a directory: {}", input.display());
        return Err(ExitCode::from(1));
    }

    let cfg = match config::load_config(args.config.as_path()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: failed to load config: {e}");
            return Err(ExitCode::from(1));
        }
    };

    let hash_algo = match cfg.hash_algorithm() {
        Ok(a) => a,
        Err(s) => {
            eprintln!("error: {s}");
            return Err(ExitCode::from(1));
        }
    };

    let threads = args.threads.unwrap_or_else(|| {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    });

    let scan_start = Instant::now();
    let paths = scanner::scan_images(input);
    let n_scanned = paths.len();
    eprintln!(
        "texture_tool: scan done: {n_scanned} paths in {:.2}s",
        scan_start.elapsed().as_secs_f64()
    );

    let mut hash_cache = file_hash::FileHashCache::new(hash_algo);

    let mut vertices: Vec<Vertex> = Vec::new();
    let ingest_start = Instant::now();
    let mut n_hash_fail = 0u64;
    let mut n_decode_ok = 0u64;
    let mut n_prepare_ok = 0u64;
    // `processed … out of …` every N images, plus once at the last image if not aligned.
    const INGEST_PROGRESS_EVERY: usize = 30;

    for (i, path) in paths.into_iter().enumerate() {
        let ingest_ix = i + 1;
        match hash_cache.digest_for_path(&path) {
            Ok(digest) => {
                let features = match ImageData::decode_path(&path, cfg.max_decode_dimension_px) {
                    Ok(ref img) => {
                        n_decode_ok += 1;
                        match prepare_vertex(img, &cfg) {
                            Ok(prepared) => {
                                n_prepare_ok += 1;
                                match extract_features(&prepared, &cfg) {
                                    Ok(f) => Some(f),
                                    Err(e) => {
                                        eprintln!("features {}: {e}", path.display());
                                        None
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("prepare {}: {e}", path.display());
                                None
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("decode: {e}");
                        None
                    }
                };
                vertices.push(Vertex {
                    path,
                    digest,
                    features,
                });
            }
            Err(e) => {
                n_hash_fail += 1;
                eprintln!("hash skip {}: {e}", path.display());
            }
        }

        if n_scanned > 0 && INGEST_PROGRESS_EVERY > 0 {
            let at_interval = ingest_ix % INGEST_PROGRESS_EVERY == 0;
            let at_end = ingest_ix == n_scanned;
            if at_interval || at_end {
                eprintln!(
                    "texture_tool: processed {ingest_ix} images out of {n_scanned}"
                );
            }
        }
    }

    eprintln!(
        "texture_tool: ingest done: {} vertices from {n_scanned} scanned; \
         decode_ok={n_decode_ok} prepare_ok={n_prepare_ok} hash_skip={n_hash_fail} \
         in {:.2}s (threads={threads})",
        vertices.len(),
        ingest_start.elapsed().as_secs_f64()
    );

    let n = vertices.len();
    let path_refs: Vec<PathBuf> = vertices.iter().map(|v| v.path.clone()).collect();
    let digests: Vec<Vec<u8>> = vertices.iter().map(|v| v.digest.clone()).collect();

    let n_pairs: u128 = if n >= 2 {
        (n as u128) * ((n - 1) as u128) / 2
    } else {
        0
    };
    eprintln!(
        "texture_tool: pairwise start: n={n} pairs={n_pairs} threads={threads}"
    );
    let pairwise_start = Instant::now();
    let stats = pairwise::pairwise_compare(&cfg, &vertices, threads);
    eprintln!(
        "texture_tool: pairwise done in {:.2}s: hash_edges={} composite_edges={} score_cache={}",
        pairwise_start.elapsed().as_secs_f64(),
        stats.hash_edges.len(),
        stats.composite_edges.len(),
        stats.score_cache.len(),
    );

    let group_indices = clustering::build_groups(
        n,
        &stats.hash_edges,
        &stats.composite_edges,
        &path_refs,
    );
    eprintln!(
        "texture_tool: clustering done: {} groups",
        group_indices.len()
    );
    let group_scores = group_score::group_scores(&group_indices, &digests, &stats.score_cache);

    let mut groups_out = Vec::with_capacity(group_indices.len());
    for (gi, members) in group_indices.iter().enumerate() {
        let id = (gi + 1) as u32;
        let mut images = Vec::with_capacity(members.len());
        for &idx in members {
            let s = canonical_path_string(&vertices[idx].path).map_err(|e| {
                eprintln!(
                    "error: could not canonicalize {}: {e}",
                    vertices[idx].path.display()
                );
                ExitCode::from(1)
            })?;
            images.push(s);
        }
        groups_out.push(output::GroupRecord {
            id,
            score: group_scores[gi],
            images,
        });
    }

    let doc = output::ResultJson {
        groups: groups_out,
    };
    eprintln!("texture_tool: writing {}", args.output.display());
    output::write_result_json(args.output.as_path(), &doc).map_err(|e| {
        eprintln!("error: writing {}: {e}", args.output.display());
        ExitCode::from(1)
    })?;

    eprintln!("wrote {}", args.output.display());
    Ok(())
}

fn canonical_path_string(p: &Path) -> Result<String, std::io::Error> {
    let c = std::fs::canonicalize(p)?;
    Ok(c.to_string_lossy().to_string())
}
