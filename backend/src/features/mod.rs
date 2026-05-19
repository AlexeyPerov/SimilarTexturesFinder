pub mod alpha_crop;
pub mod extract;

#[derive(Clone, Debug)]
pub struct ImageFeatures {
    pub phash: u64,
    pub ssim_luma: Vec<f32>,
    pub histogram: Vec<f32>,
}

#[derive(Clone, Debug)]
pub struct Features {
    /// Index 0 is always the identity transform.
    /// Additional entries are rotation/flip transforms when enabled.
    pub transforms: Vec<ImageFeatures>,
}
