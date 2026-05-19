mod cancel;
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

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use clap::Parser;

use crate::cancel::{is_cancelled, setup_cancel_handler};
use crate::features::extract::extract_features;
use crate::image_loader::ImageData;
use crate::similarity::prepare_vertex;
use crate::vertex::Vertex;

const EXIT_CANCELLED: u8 = 2;

#[derive(Parser, Debug)]
#[command(name = "texture_tool", version, about = "Similar texture finder (MVP backend)")]
struct Args {
    #[arg(long)]
    input: PathBuf,

    #[arg(long)]
    output: PathBuf,

    #[arg(long)]
    config: PathBuf,

    #[arg(long)]
    threads: Option<usize>,
}

fn main() -> ExitCode {
    setup_cancel_handler();
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

    if is_cancelled() {
        eprintln!("texture_tool: cancelled");
        return Err(ExitCode::from(EXIT_CANCELLED));
    }

    let mut hash_cache = file_hash::FileHashCache::new(hash_algo);

    let mut vertices: Vec<Vertex> = Vec::new();
    let ingest_start = Instant::now();
    let mut n_hash_fail = 0u64;
    let mut n_decode_ok = 0u64;
    let mut n_prepare_ok = 0u64;
    const INGEST_PROGRESS_EVERY: usize = 30;

    for (i, path) in paths.into_iter().enumerate() {
        if is_cancelled() {
            eprintln!("texture_tool: cancelled during ingestion at image {}/{n_scanned}", i + 1);
            return Err(ExitCode::from(EXIT_CANCELLED));
        }

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

    if is_cancelled() {
        eprintln!("texture_tool: cancelled before pairwise");
        return Err(ExitCode::from(EXIT_CANCELLED));
    }

    let n = vertices.len();

    let (hash_edges, digest_rep) = build_hash_groups(&vertices);
    if !hash_edges.is_empty() {
        let n_dup_groups = digest_rep
            .iter()
            .enumerate()
            .filter(|(i, r)| **r != *i)
            .count();
        eprintln!(
            "texture_tool: {} hash-equal pairs from {n_dup_groups} duplicate vertices",
            hash_edges.len(),
        );
    }

    let n_pairs: u128 = if n >= 2 {
        (n as u128) * ((n - 1) as u128) / 2
    } else {
        0
    };
    let n_hash_pairs = hash_edges.len() as u128;
    eprintln!(
        "texture_tool: pairwise start: n={n} total_pairs={n_pairs} hash_pairs={n_hash_pairs} threads={threads}"
    );
    let pairwise_start = Instant::now();
    let stats = pairwise::pairwise_compare(&cfg, &vertices, threads, &digest_rep);

    if is_cancelled() {
        eprintln!("texture_tool: cancelled during pairwise");
        return Err(ExitCode::from(EXIT_CANCELLED));
    }

    eprintln!(
        "texture_tool: pairwise done in {:.2}s: hash_edges={} composite_edges={} score_cache={}",
        pairwise_start.elapsed().as_secs_f64(),
        hash_edges.len(),
        stats.composite_edges.len(),
        stats.score_cache.len(),
    );

    let group_indices = clustering::build_groups(
        n,
        &hash_edges,
        &stats.composite_edges,
        &vertices,
    );
    eprintln!(
        "texture_tool: clustering done: {} groups",
        group_indices.len()
    );
    let group_scores = group_score::group_scores(&group_indices, &vertices, &stats.score_cache);

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
