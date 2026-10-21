use std::fs;
use std::io;
use std::num::NonZeroUsize;
use std::panic;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use v_ide_editor::path::rel_path;

use crate::note::Note;

type Parsed = (usize, io::Result<Note>);

pub fn scan(root: &Path) -> io::Result<Vec<Note>> {
    scan_with(root, false)
}

pub fn scan_with(root: &Path, include_hidden: bool) -> io::Result<Vec<Note>> {
    let mut paths = Vec::new();
    walk(root, include_hidden, &mut paths)?;
    let mut notes = parse_all(root, &paths)?;
    notes.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(notes)
}

fn walk(dir: &Path, include_hidden: bool, paths: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        if !include_hidden && name.to_string_lossy().starts_with('.') {
            continue;
        }
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            walk(&path, include_hidden, paths)?;
        } else if path.extension().is_some_and(|ext| ext == "md") {
            paths.push(path);
        }
    }
    Ok(())
}

fn parse_all(root: &Path, paths: &[PathBuf]) -> io::Result<Vec<Note>> {
    let next = AtomicUsize::new(0);
    let limit = AtomicUsize::new(paths.len());
    let workers = thread::available_parallelism()
        .map_or(1, NonZeroUsize::get)
        .min(paths.len());
    let mut parsed: Vec<Parsed> = thread::scope(|scope| {
        let handles: Vec<_> = (0..workers)
            .map(|_| scope.spawn(|| parse_queue(root, paths, &next, &limit)))
            .collect();
        handles
            .into_iter()
            .flat_map(|handle| {
                handle
                    .join()
                    .unwrap_or_else(|err| panic::resume_unwind(err))
            })
            .collect()
    });
    parsed.sort_by_key(|(at, _)| *at);
    parsed.into_iter().map(|(_, note)| note).collect()
}

fn parse_queue(
    root: &Path,
    paths: &[PathBuf],
    next: &AtomicUsize,
    limit: &AtomicUsize,
) -> Vec<Parsed> {
    let mut parsed = Vec::new();
    loop {
        let at = next.fetch_add(1, Ordering::Relaxed);
        let Some(path) = paths.get(at).filter(|_| at < limit.load(Ordering::Relaxed)) else {
            return parsed;
        };
        let note = read_note(root, path);
        if note.is_err() {
            limit.fetch_min(at, Ordering::Relaxed);
        }
        parsed.push((at, note));
    }
}

fn read_note(root: &Path, path: &Path) -> io::Result<Note> {
    let source = fs::read_to_string(path)?;
    Ok(Note::from_source(&rel_path(root, path), &source))
}
