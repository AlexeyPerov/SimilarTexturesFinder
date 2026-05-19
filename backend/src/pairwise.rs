use std::collections::HashMap;

use rayon::prelude::*;

use crate::config::Config;
use crate::features::Features;
use crate::similarity::histogram::{self, HistMethod};
use crate::similarity::phash;
use crate::similarity::ssim;
use crate::similarity::types::MetricResult;
use crate::similarity::combine;
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
    let features: Vec<Option<Features>> = vertices.iter().map(|v| v.features.clone()).collect();

    let hist_method = histogram::parse_hist_method(&cfg.hist_method);

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
                } else if let (Some(fa), Some(fb)) = (&features[i], &features[j]) {
                    let ph = cfg
                        .enable_phash
                        .then(|| max_phash(fa, fb, cfg.phash_max_distance));
                    let ss = cfg
                        .enable_ssim
                        .then(|| max_ssim(fa, fb, cfg.ssim_threshold));
                    let hi = cfg
                        .enable_histogram
                        .then(|| max_hist(fa, fb, hist_method));
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

fn max_phash(fa: &Features, fb: &Features, max_dist: u32) -> MetricResult {
    let ha = fa.transforms[0].phash;
    let mut best = MetricResult {
        score: -1.0,
        raw: 0.0,
        valid: false,
    };
    for tb in &fb.transforms {
        let r = phash::score_hashes(ha, tb.phash, max_dist);
        if r.valid && r.score > best.score {
            best = r;
        }
    }
    if !best.valid {
        MetricResult {
            score: 0.0,
            raw: 0.0,
            valid: false,
        }
    } else {
        best
    }
}

fn max_ssim(fa: &Features, fb: &Features, min_ssim: f64) -> MetricResult {
    let la = &fa.transforms[0].ssim_luma;
    let mut best = MetricResult {
        score: -1.0,
        raw: 0.0,
        valid: false,
    };
    for tb in &fb.transforms {
        let r = ssim::score_luma(la, &tb.ssim_luma, min_ssim);
        if r.valid && r.score > best.score {
            best = r;
        }
    }
    if !best.valid {
        MetricResult {
            score: 0.0,
            raw: 0.0,
            valid: false,
        }
    } else {
        best
    }
}

fn max_hist(fa: &Features, fb: &Features, method: HistMethod) -> MetricResult {
    let ha = &fa.transforms[0].histogram;
    let mut best = MetricResult {
        score: -1.0,
        raw: 0.0,
        valid: false,
    };
    for tb in &fb.transforms {
        let r = histogram::score_hist(ha, &tb.histogram, method);
        if r.valid && r.score > best.score {
            best = r;
        }
    }
    if !best.valid {
        MetricResult {
            score: 0.0,
            raw: 0.0,
            valid: false,
        }
    } else {
        best
    }
}
