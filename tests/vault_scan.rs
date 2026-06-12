use std::fs;
use std::path::PathBuf;

use vi_book::vault::cache::{self, Cache, Entry};
use vi_book::vault::note::Note;
use vi_book::vault::scan::scan;

struct TempVault {
    root: PathBuf,
}

impl TempVault {
    fn new(name: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("vi-book-vault-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        Self { root }
    }

    fn write(&self, rel: &str, content: &str) {
        let path = self.root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }
}

impl Drop for TempVault {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn paths(notes: &[Note]) -> Vec<String> {
    notes.iter().map(|n| n.path.clone()).collect()
}

#[test]
fn scan_finds_md_files_recursively_with_relative_paths() {
    let vault = TempVault::new("scan");
    vault.write("a.md", "[[b]]");
    vault.write("docs/b.md", "# B");
    vault.write("docs/notes/c.md", "x");
    vault.write("readme.txt", "ignored");
    let notes = scan(&vault.root).unwrap();
    assert_eq!(paths(&notes), vec!["a.md", "docs/b.md", "docs/notes/c.md"]);
    assert_eq!(notes[0].links[0].target, "b");
}

#[test]
fn scan_skips_hidden_directories() {
    let vault = TempVault::new("hidden");
    vault.write("a.md", "x");
    vault.write(".git/b.md", "x");
    vault.write(".obsidian/c.md", "x");
    assert_eq!(paths(&scan(&vault.root).unwrap()), vec!["a.md"]);
}

#[test]
fn cache_roundtrips_through_disk() {
    let vault = TempVault::new("roundtrip");
    let cache = Cache {
        entries: vec![Entry {
            note: Note::from_source("a.md", "[[b]] #t"),
            mtime: 42,
        }],
    };
    let path = vault.root.join("index.json");
    cache::save(&path, &cache).unwrap();
    let loaded = cache::load(&path).unwrap();
    assert_eq!(loaded.entries, cache.entries);
}

#[test]
fn load_missing_cache_returns_none() {
    let vault = TempVault::new("missing");
    assert!(cache::load(&vault.root.join("nope.json")).is_none());
}

#[test]
fn reconcile_reuses_cached_entry_when_mtime_matches() {
    let vault = TempVault::new("reuse");
    vault.write("a.md", "[[fresh]]");
    let mtime = cache::mtime_of(&vault.root.join("a.md")).unwrap();
    let stale = Cache {
        entries: vec![Entry {
            note: Note::from_source("a.md", "[[stale]]"),
            mtime,
        }],
    };
    let result = cache::reconcile(&vault.root, &stale).unwrap();
    assert_eq!(result.entries[0].note.links[0].target, "stale");
}

#[test]
fn reconcile_reparses_when_mtime_differs() {
    let vault = TempVault::new("reparse");
    vault.write("a.md", "[[fresh]]");
    let stale = Cache {
        entries: vec![Entry {
            note: Note::from_source("a.md", "[[stale]]"),
            mtime: 0,
        }],
    };
    let result = cache::reconcile(&vault.root, &stale).unwrap();
    assert_eq!(result.entries[0].note.links[0].target, "fresh");
}

#[test]
fn reconcile_adds_new_and_drops_deleted() {
    let vault = TempVault::new("delta");
    vault.write("kept.md", "x");
    vault.write("added.md", "y");
    let previous = Cache {
        entries: vec![
            Entry {
                note: Note::from_source("kept.md", "x"),
                mtime: 0,
            },
            Entry {
                note: Note::from_source("deleted.md", "z"),
                mtime: 0,
            },
        ],
    };
    let result = cache::reconcile(&vault.root, &previous).unwrap();
    let mut names: Vec<&str> = result
        .entries
        .iter()
        .map(|e| e.note.path.as_str())
        .collect();
    names.sort();
    assert_eq!(names, vec!["added.md", "kept.md"]);
}

#[test]
fn cache_path_joins_vault_name() {
    let path = cache::cache_path(std::path::Path::new("/cache"), "myvault");
    assert!(path.ends_with("myvault.json"));
}
