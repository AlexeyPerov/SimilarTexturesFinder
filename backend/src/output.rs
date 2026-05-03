use std::io::Write;
use std::path::Path;

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

/// Write JSON atomically: temp in target directory, sync, rename ([Task.md](../../../Tasks/Task.md) §3.9 SHOULD).
pub fn write_result_json(path: &Path, doc: &ResultJson) -> std::io::Result<()> {
    let dir = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));

    let mut tmp = tempfile::NamedTempFile::new_in(dir)?;
    {
        let mut buf = Vec::new();
        serde_json::to_writer_pretty(&mut buf, doc)?;
        tmp.write_all(&buf)?;
    }
    tmp.as_file().sync_all()?;
    tmp.persist(path)
        .map_err(|e| e.error)
        .map(|_| ())
}
