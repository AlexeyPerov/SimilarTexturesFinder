use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use digest::Digest;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HashAlgorithm {
    Md5,
    Sha1,
    Sha256,
}

impl HashAlgorithm {
    pub fn parse(raw: &str) -> Result<Self, String> {
        match raw.to_ascii_lowercase().as_str() {
            "md5" => Ok(Self::Md5),
            "sha1" => Ok(Self::Sha1),
            "sha256" => Ok(Self::Sha256),
            _ => Err(raw.to_string()),
        }
    }
}

pub struct FileHashCache {
    algo: HashAlgorithm,
    digests: HashMap<std::path::PathBuf, Vec<u8>>,
}

impl FileHashCache {
    pub fn new(algo: HashAlgorithm) -> Self {
        Self {
            algo,
            digests: HashMap::new(),
        }
    }

    /// Streaming hash; caches per absolute path key.
    pub fn digest_for_path(&mut self, path: &Path) -> std::io::Result<Vec<u8>> {
        let key = path.to_path_buf();
        if let Some(d) = self.digests.get(&key).cloned() {
            return Ok(d);
        }
        let d = hash_file(path, self.algo)?;
        self.digests.insert(key, d.clone());
        Ok(d)
    }
}

pub fn hash_file(path: &Path, algo: HashAlgorithm) -> std::io::Result<Vec<u8>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut buf = [0u8; 64 * 1024];

    match algo {
        HashAlgorithm::Md5 => {
            let mut hasher = md5::Md5::new();
            loop {
                let n = reader.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                hasher.update(&buf[..n]);
            }
            Ok(hasher.finalize().to_vec())
        }
        HashAlgorithm::Sha1 => {
            let mut hasher = sha1::Sha1::new();
            loop {
                let n = reader.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                hasher.update(&buf[..n]);
            }
            Ok(hasher.finalize().to_vec())
        }
        HashAlgorithm::Sha256 => {
            let mut hasher = sha2::Sha256::new();
            loop {
                let n = reader.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                hasher.update(&buf[..n]);
            }
            Ok(hasher.finalize().to_vec())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn identical_files_same_digest() {
        let dir = tempdir().unwrap();
        let a = dir.path().join("a.bin");
        let b = dir.path().join("b.bin");
        let data = b"hello world";
        std::fs::File::create(&a).unwrap().write_all(data).unwrap();
        std::fs::File::create(&b).unwrap().write_all(data).unwrap();

        let da = hash_file(&a, HashAlgorithm::Sha256).unwrap();
        let db = hash_file(&b, HashAlgorithm::Sha256).unwrap();
        assert_eq!(da, db);
    }
}
