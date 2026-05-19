use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

const ALLOWED_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "webp", "bmp", "gif", "tiff", "tif",
];

/// Recursively collect image paths under `root`. Symlinks are not followed;
/// paths are canonicalized and deduplicated to prevent the same physical file
/// from appearing as multiple vertices. Sorted lexicographically by lossy UTF-8.
pub fn scan_images(root: &Path) -> Vec<PathBuf> {
    let mut raw = Vec::new();
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

        raw.push(path.to_path_buf());
    }

    let mut seen = HashSet::new();
    let mut out = Vec::with_capacity(raw.len());
    for path in raw {
        match std::fs::canonicalize(&path) {
            Ok(c) => {
                if seen.insert(c.clone()) {
                    out.push(c);
                }
            }
            Err(e) => {
                eprintln!(
                    "scanner: skip (canonicalize): {}: {e}",
                    path.display()
                );
            }
        }
    }

    let mut tagged: Vec<(String, PathBuf)> = out
        .into_iter()
        .map(|p| (p.to_string_lossy().into_owned(), p))
        .collect();
    tagged.sort_by(|a, b| a.0.cmp(&b.0));
    tagged.into_iter().map(|(_, p)| p).collect()
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
        assert!(paths[0].to_string_lossy().ends_with("a.jpg"));
        assert!(paths[1].to_string_lossy().ends_with("b.png"));
    }

    #[test]
    fn canonicalizes_paths() {
        let dir = tempdir().unwrap();
        let sub = dir.path().join("sub");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("a.png"), b"hello").unwrap();

        let link = dir.path().join("link");
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&sub, &link).unwrap();
        }

        let paths_main = scan_images(&sub);
        assert_eq!(paths_main.len(), 1);

        #[cfg(unix)]
        {
            let paths_link = scan_images(&link);
            assert_eq!(paths_link.len(), 1);
        }
    }
}
