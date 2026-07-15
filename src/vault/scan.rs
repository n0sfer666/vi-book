use std::fs;
use std::io;
use std::path::Path;

use crate::vault::note::Note;

pub fn scan(root: &Path) -> io::Result<Vec<Note>> {
    scan_with(root, false)
}

pub fn scan_with(root: &Path, include_hidden: bool) -> io::Result<Vec<Note>> {
    let mut notes = Vec::new();
    walk(root, root, include_hidden, &mut notes)?;
    notes.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(notes)
}

pub fn rel_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn walk(root: &Path, dir: &Path, include_hidden: bool, notes: &mut Vec<Note>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        if !include_hidden && name.to_string_lossy().starts_with('.') {
            continue;
        }
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            walk(root, &path, include_hidden, notes)?;
        } else if path.extension().is_some_and(|ext| ext == "md") {
            let source = fs::read_to_string(&path)?;
            notes.push(Note::from_source(&rel_path(root, &path), &source));
        }
    }
    Ok(())
}
