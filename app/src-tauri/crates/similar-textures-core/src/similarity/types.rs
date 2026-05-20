use crate::image_loader::ImageData;

/// Normalized similarity in `[0, 1]` when `valid` is true ([`Task_similarity_appendix.md`](../../Tasks/Task_similarity_appendix.md) §A).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MetricResult {
    pub score: f32,
    pub raw: f32,
    pub valid: bool,
}

pub trait SimilarityMetric {
    fn compute(&self, img_a: &ImageData, img_b: &ImageData) -> MetricResult;
}
