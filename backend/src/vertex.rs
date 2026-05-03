use std::path::PathBuf;

use crate::image_loader::ImageData;

pub struct Vertex {
    pub path: PathBuf,
    pub digest: Vec<u8>,
    pub prepared: Option<ImageData>,
}
