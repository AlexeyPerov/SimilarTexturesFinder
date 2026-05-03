use std::collections::HashMap;

use rayon::prelude::*;

use crate::config::Config;
use crate::similarity::{combine, max_over_b, HistogramMetric, PHashMetric, SsimMetric};
use crate::vertex::Vertex;

pub struct PairwiseStats {
    pub score_cache: HashMap<(usize, usize), f64>,
    pub hash_edges: Vec<(usize, usize)>,
    pub composite_edges: Vec<(usize, usize)>,
}

fn pair_key(i: usize, j: usize) -> (usize, usize) {
    if i < j {
        (i, j)
    } else {
        (j, i)
    }
}

pub fn pairwise_compare(cfg: &Config, vertices: &[Vertex], threads: usize) -> PairwiseStats {
    let n = vertices.len();
    let digests: Vec<Vec<u8>> = vertices.iter().map(|v| v.digest.clone()).collect();
    let prepared: Vec<Option<crate::image_loader::ImageData>> =
        vertices.iter().map(|v| v.prepared.clone()).collect();

    let ph_m = PHashMetric::from_config(cfg);
    let ss_m = SsimMetric::from_config(cfg);
    let hi_m = HistogramMetric::from_config(cfg);

    let pairs: Vec<(usize, usize)> = (0..n)
        .flat_map(|i| ((i + 1)..n).map(move |j| (i, j)))
        .collect();

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads.max(1))
        .build()
        .expect("rayon thread pool");

    let thr = cfg.threshold;
    let results: Vec<(usize, usize, bool, Option<f64>)> = pool.install(|| {
        pairs
            .par_iter()
            .map(|&(i, j)| {
                let hash_eq = digests[i] == digests[j];
                let fs_out = if hash_eq {
                    None
                } else if let (Some(a), Some(b)) = (&prepared[i], &prepared[j]) {
                    let ph = cfg
                        .enable_phash
                        .then(|| max_over_b(&ph_m, a, b, cfg));
                    let ss = cfg
                        .enable_ssim
                        .then(|| max_over_b(&ss_m, a, b, cfg));
                    let hi = cfg
                        .enable_histogram
                        .then(|| max_over_b(&hi_m, a, b, cfg));
                    combine(cfg, ph, ss, hi)
                } else {
                    None
                };

                (i, j, hash_eq, fs_out)
            })
            .collect()
    });

    let mut score_cache = HashMap::new();
    let mut hash_edges = Vec::new();
    let mut composite_edges = Vec::new();

    for (i, j, hash_eq, fs) in results {
        if hash_eq {
            hash_edges.push((i, j));
        }
        if let Some(fs) = fs {
            score_cache.insert(pair_key(i, j), fs);
            if !hash_eq && fs > thr {
                composite_edges.push((i, j));
            }
        }
    }

    PairwiseStats {
        score_cache,
        hash_edges,
        composite_edges,
    }
}