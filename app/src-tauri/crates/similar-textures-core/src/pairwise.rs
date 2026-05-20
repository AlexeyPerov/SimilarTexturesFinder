use std::collections::HashMap;

use rayon::prelude::*;

use crate::config::Config;
use crate::ScanCallbacks;
use crate::similarity::combine;
use crate::similarity::histogram::{self, HistMethod};
use crate::similarity::phash;
use crate::similarity::ssim;
use crate::similarity::types::MetricResult;
use crate::vertex::Vertex;

pub struct PairwiseStats {
    pub score_cache: HashMap<(usize, usize), f64>,
    pub composite_edges: Vec<(usize, usize)>,
}

fn pair_key(i: usize, j: usize) -> (usize, usize) {
    if i < j {
        (i, j)
    } else {
        (j, i)
    }
}

pub fn pairwise_compare(
    cfg: &Config,
    vertices: &[Vertex],
    threads: usize,
    digest_rep: &[usize],
    callbacks: &dyn ScanCallbacks,
) -> PairwiseStats {
    let n = vertices.len();
    let hist_method = histogram::parse_hist_method(&cfg.hist_method);
    let thr = cfg.threshold;

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads.max(1))
        .build()
        .expect("rayon thread pool");

    let results: Vec<(usize, usize, Option<f64>)> = pool.install(|| {
        (0..n)
            .into_par_iter()
            .flat_map(|i| {
                let rep_i = digest_rep[i];
                (i + 1..n)
                    .into_par_iter()
                    .filter_map(move |j| {
                        if callbacks.is_cancelled() {
                            return None;
                        }
                        if rep_i == digest_rep[j] {
                            return None;
                        }
                        let fs_out =
                            if let (Some(fa), Some(fb)) =
                                (&vertices[i].features, &vertices[j].features)
                            {
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

                        Some((i, j, fs_out))
                    })
            })
            .collect()
    });

    let mut score_cache = HashMap::new();
    let mut composite_edges = Vec::new();

    for (i, j, fs) in results {
        if let Some(fs) = fs {
            if fs > thr {
                composite_edges.push((i, j));
                score_cache.insert(pair_key(i, j), fs);
            }
        }
    }

    PairwiseStats {
        score_cache,
        composite_edges,
    }
}

fn max_phash(
    fa: &crate::features::Features,
    fb: &crate::features::Features,
    max_dist: u32,
) -> MetricResult {
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

fn max_ssim(
    fa: &crate::features::Features,
    fb: &crate::features::Features,
    min_ssim: f64,
) -> MetricResult {
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

fn max_hist(
    fa: &crate::features::Features,
    fb: &crate::features::Features,
    method: HistMethod,
) -> MetricResult {
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
