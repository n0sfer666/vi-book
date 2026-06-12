use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};

use crate::vault::note::Note;
use crate::vault::scan;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub note: Note,
    pub mtime: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Cache {
    pub entries: Vec<Entry>,
}

impl Cache {
    pub fn notes(&self) -> Vec<Note> {
        self.entries.iter().map(|e| e.note.clone()).collect()
    }
}

pub fn cache_path(cache_root: &Path, vault_name: &str) -> PathBuf {
    cache_root.join(format!("{vault_name}.json"))
}

pub fn load(path: &Path) -> Option<Cache> {
    let text = fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

pub fn save(path: &Path, cache: &Cache) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text =
        serde_json::to_string(cache).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(path, text)
}

pub fn mtime_of(path: &Path) -> io::Result<u64> {
    let modified = fs::metadata(path)?.modified()?;
    Ok(modified
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0))
}

pub fn reconcile(root: &Path, previous: &Cache) -> io::Result<Cache> {
    let mut cached: HashMap<&str, &Entry> = previous
        .entries
        .iter()
        .map(|e| (e.note.path.as_str(), e))
        .collect();
    let mut entries = Vec::new();
    let mut files = Vec::new();
    collect_md(root, root, &mut files)?;
    for (rel, abs) in files {
        let mtime = mtime_of(&abs)?;
        let entry = match cached.remove(rel.as_str()) {
            Some(prev) if prev.mtime == mtime => prev.clone(),
            _ => Entry {
                note: Note::from_source(&rel, &fs::read_to_string(&abs)?),
                mtime,
            },
        };
        entries.push(entry);
    }
    entries.sort_by(|a, b| a.note.path.cmp(&b.note.path));
    Ok(Cache { entries })
}

fn collect_md(root: &Path, dir: &Path, files: &mut Vec<(String, PathBuf)>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_name().to_string_lossy().starts_with('.') {
            continue;
        }
        if entry.file_type()?.is_dir() {
            collect_md(root, &path, files)?;
        } else if path.extension().is_some_and(|ext| ext == "md") {
            files.push((scan::rel_path(root, &path), path));
        }
    }
    Ok(())
}
