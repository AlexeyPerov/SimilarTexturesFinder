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

use clap::Parser;

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

    let paths = scanner::scan_images(input);
    let n_scanned = paths.len();
    let mut hash_cache = file_hash::FileHashCache::new(hash_algo);

    let mut vertices: Vec<Vertex> = Vec::new();
    for path in paths {
        match hash_cache.digest_for_path(&path) {
            Ok(digest) => {
                let prepared = match ImageData::decode_path(&path, cfg.max_decode_dimension_px) {
                    Ok(ref img) => match prepare_vertex(img, &cfg) {
                        Ok(p) => Some(p),
                        Err(e) => {
                            eprintln!("prepare {}: {e}", path.display());
                            None
                        }
                    },
                    Err(e) => {
                        eprintln!("decode: {e}");
                        None
                    }
                };
                vertices.push(Vertex {
                    path,
                    digest,
                    prepared,
                });
            }
            Err(e) => {
                eprintln!("hash skip {}: {e}", path.display());
            }
        }
    }

    eprintln!(
        "texture_tool: {} vertices from {} scanned paths (threads={threads})",
        vertices.len(),
        n_scanned,
    );

    let n = vertices.len();
    let path_refs: Vec<PathBuf> = vertices.iter().map(|v| v.path.clone()).collect();
    let digests: Vec<Vec<u8>> = vertices.iter().map(|v| v.digest.clone()).collect();

    let stats = pairwise::pairwise_compare(&cfg, &vertices, threads);
    let group_indices = clustering::build_groups(
        n,
        &stats.hash_edges,
        &stats.composite_edges,
        &path_refs,
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
