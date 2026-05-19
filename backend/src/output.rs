use std::io::Write;
use std::path::Path;
use std::thread;
use std::time::Duration;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ResultJson {
    pub groups: Vec<GroupRecord>,
}

#[derive(Debug, Serialize)]
pub struct GroupRecord {
    pub id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
    pub images: Vec<String>,
}

/// Write JSON atomically: temp file in target directory, sync, rename.
/// On Windows, retries the rename up to 3 times (the target may be briefly
/// open by another process). Falls back to a non-atomic direct write on
/// persistent failure.
pub fn write_result_json(path: &Path, doc: &ResultJson) -> std::io::Result<()> {
    let dir = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));

    let mut buf = Vec::new();
    serde_json::to_writer_pretty(&mut buf, doc)?;

    let mut tmp = tempfile::NamedTempFile::new_in(dir)?;
    tmp.write_all(&buf)?;
    tmp.as_file().sync_all()?;

    if let Err(persist_err) = tmp.persist(path) {
        let mut last_err = persist_err.error;
        let mut tempfile = persist_err.file;

        for _ in 0..3 {
            thread::sleep(Duration::from_millis(50));
            match tempfile.persist(path) {
                Ok(_) => return Ok(()),
                Err(e) => {
                    last_err = e.error;
                    tempfile = e.file;
                }
            }
        }

        eprintln!(
            "warn: atomic rename failed after retries, falling back to direct write: {last_err}"
        );
        drop(tempfile);
        let mut f = std::fs::File::create(path)?;
        f.write_all(&buf)?;
        f.sync_all()?;
    }

    Ok(())
}
