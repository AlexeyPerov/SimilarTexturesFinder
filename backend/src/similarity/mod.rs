//! Similarity metrics: shared result type and [`SimilarityMetric`] trait
//! ([Task_similarity_appendix.md](../../Tasks/Task_similarity_appendix.md) §A–§B).
//!
//! When `MetricResult::valid` is false, `score` is not meaningful. When `valid` is true,
//! `score` is a similarity in `[0.0, 1.0]` (1.0 = identical per method).

#![allow(unused_imports)]

mod composite;
mod histogram;
mod phash;
mod prepare;
mod rotations;
mod ssim;
mod types;

pub use crate::image_loader::ImageData;
pub use composite::combine;
pub use histogram::HistogramMetric;
pub use phash::PHashMetric;
pub use prepare::prepare_vertex;
pub use rotations::max_over_b;
pub use ssim::SsimMetric;
pub use types::{MetricResult, SimilarityMetric};
