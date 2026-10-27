mod common;

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use v_ide_vault::watcher::{Change, Watcher};

#[test]
fn watcher_reports_modified_file_path() {
    let root = common::scratch("watch");
    let file = root.join("note.md");
    fs::write(&file, "one").unwrap();

    let watcher = Watcher::new(&root).unwrap();
    fs::write(&file, "two").unwrap();

    let mut seen: Option<PathBuf> = None;
    let deadline = Duration::from_secs(5);
    let start = std::time::Instant::now();
    while start.elapsed() < deadline {
        if let Some(Change::Path(path)) = watcher.poll(Duration::from_millis(200))
            && path.file_name().is_some_and(|n| n == "note.md")
        {
            seen = Some(path);
            break;
        }
    }
    let _ = fs::remove_dir_all(&root);
    assert!(seen.is_some(), "watcher did not report the modified file");
}
