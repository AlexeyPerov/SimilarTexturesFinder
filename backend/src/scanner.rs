use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

const ALLOWED_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "webp", "bmp", "gif", "tiff", "tif",
];

/// Recursively collect image paths under `root`. Symlinks are not followed (directories are not descended via symlinks; symlink files are skipped when detected).
/// Paths are sorted lexicographically by their UTF-8 lossy string for stable ordering.
pub fn scan_images(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let walker = WalkDir::new(root).follow_links(false).into_iter();
    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("scanner: skip (walk error): {e}");
                continue;
            }
        };

        let path = entry.path();
        let ft = entry.file_type();

        if ft.is_symlink() {
            continue;
        }

        if !ft.is_file() {
            continue;
        }

        if !extension_allowed(path) {
            continue;
        }

        out.push(path.to_path_buf());
    }

    out.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));
    out
}

fn extension_allowed(path: &Path) -> bool {
    let Some(ext) = path.extension().and_then(OsStr::to_str) else {
        return false;
    };
    let ext_lower = ext.to_ascii_lowercase();
    ALLOWED_EXTENSIONS.iter().any(|&e| e == ext_lower)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn sorts_and_filters_extensions() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("b.png"), []).unwrap();
        fs::write(dir.path().join("a.jpg"), []).unwrap();
        fs::write(dir.path().join("skip.txt"), []).unwrap();

        let paths = scan_images(dir.path());
        assert_eq!(paths.len(), 2);
        assert!(paths[0].ends_with("a.jpg"));
        assert!(paths[1].ends_with("b.png"));
    }
}
