#![allow(unused_imports)]

pub(crate) mod composite;
pub(crate) mod histogram;
pub(crate) mod phash;
pub(crate) mod prepare;
pub(crate) mod rotations;
pub(crate) mod ssim;
pub(crate) mod types;

pub use crate::image_loader::ImageData;
pub use composite::combine;
pub use histogram::HistogramMetric;
pub use phash::PHashMetric;
pub use prepare::prepare_vertex;
pub use rotations::max_over_b;
pub use ssim::SsimMetric;
pub use types::{MetricResult, SimilarityMetric};
