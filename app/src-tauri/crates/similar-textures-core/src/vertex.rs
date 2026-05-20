use std::path::PathBuf;

use crate::features::Features;

pub struct Vertex {
    pub path: PathBuf,
    pub digest: Vec<u8>,
    pub features: Option<Features>,
}
