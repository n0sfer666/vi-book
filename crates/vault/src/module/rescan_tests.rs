use std::fs;

use crate::watcher::Change;

use super::batch::Batch;
use super::disk;
use super::lazy::LazyIndex;
use super::test_vault::{rescan, vault};

#[test]
fn rescan_rebuilds_facts() {
    let root = vault("rescan-facts");
    let index = LazyIndex::new(root.clone());
    assert_eq!(index.get().unwrap().tags(), vec![String::from("idea")]);
    fs::write(root.join("a.md"), "# A #fresh\n").unwrap();
    let applied = disk::apply(&index, &rescan());
    assert!(applied.changed);
    assert!(applied.errors.is_empty(), "{:?}", applied.errors);
    assert_eq!(
        index.get().unwrap().tags(),
        vec![String::from("fresh"), String::from("idea")]
    );
}

#[test]
fn rescan_with_names_only_forgets_names() {
    let root = vault("rescan-names");
    let index = LazyIndex::new(root.clone());
    assert_eq!(index.paths().unwrap().len(), 2);
    fs::write(root.join("c.md"), "# C\n").unwrap();
    let applied = disk::apply(&index, &rescan());
    assert!(!applied.changed);
    assert!(!index.is_built());
    assert_eq!(index.paths().unwrap().len(), 3);
}

#[test]
fn busy_index_defers_full_rescan() {
    let root = vault("busy-rescan");
    let index = LazyIndex::new(root);
    let held = index.get().unwrap();
    let applied = disk::apply(&index, &rescan());
    drop(held);
    assert!(!applied.changed);
    assert_eq!(applied.errors.len(), 1);
    assert!(index.watch().drain().rescan);
}

#[test]
fn failed_rescan_keeps_facts() {
    let root = vault("rescan-unreadable");
    let index = LazyIndex::new(root.clone());
    assert_eq!(index.get().unwrap().tags(), vec![String::from("idea")]);
    fs::write(root.join("a.md"), [0xff, 0xfe, 0x00]).unwrap();
    let applied = disk::apply(&index, &rescan());
    assert!(!applied.changed);
    assert_eq!(applied.errors.len(), 1);
    assert_eq!(index.get().unwrap().tags(), vec![String::from("idea")]);
    assert!(!index.watch().pending());
}

#[test]
fn failed_rescan_retries_on_next_change() {
    let root = vault("rescan-retry");
    let index = LazyIndex::new(root.clone());
    assert_eq!(index.get().unwrap().tags(), vec![String::from("idea")]);
    fs::write(root.join("a.md"), [0xff, 0xfe, 0x00]).unwrap();
    assert_eq!(disk::apply(&index, &rescan()).errors.len(), 1);
    fs::write(root.join("a.md"), "# A #fixed\n").unwrap();
    fs::write(root.join("c.md"), "# C\n").unwrap();
    let mut batch = Batch::default();
    index
        .watch()
        .absorb(&mut batch, Change::Path(root.join("a.md")));
    assert!(batch.rescan);
    let applied = disk::apply(&index, &batch);
    assert!(applied.changed);
    assert_eq!(index.get().unwrap().notes().len(), 3);
}

#[test]
fn repeated_rescan_failure_reports_once() {
    let root = vault("rescan-once");
    let index = LazyIndex::new(root.clone());
    assert!(index.get().is_ok());
    fs::write(root.join("a.md"), [0xff, 0xfe, 0x00]).unwrap();
    assert_eq!(disk::apply(&index, &rescan()).errors.len(), 1);
    assert_eq!(disk::apply(&index, &rescan()).errors, Vec::<String>::new());
}

#[cfg(unix)]
#[test]
fn vanished_during_rescan_retries_next_tick() {
    let root = vault("rescan-vanished");
    let index = LazyIndex::new(root.clone());
    assert_eq!(index.get().unwrap().tags(), vec![String::from("idea")]);
    std::os::unix::fs::symlink(root.join("gone.md"), root.join("x.md")).unwrap();
    let applied = disk::apply(&index, &rescan());
    assert!(!applied.changed);
    assert_eq!(applied.errors, Vec::<String>::new());
    assert_eq!(index.get().unwrap().tags(), vec![String::from("idea")]);
    assert!(index.watch().drain().rescan);
}

#[cfg(unix)]
#[test]
fn persistent_vanished_reports_once_and_waits_for_change() {
    let root = vault("rescan-dangling");
    let index = LazyIndex::new(root.clone());
    assert!(index.get().is_ok());
    std::os::unix::fs::symlink(root.join("gone.md"), root.join("x.md")).unwrap();
    assert_eq!(disk::apply(&index, &rescan()).errors, Vec::<String>::new());
    assert!(index.watch().drain().rescan);
    assert_eq!(disk::apply(&index, &rescan()).errors.len(), 1);
    assert!(!index.watch().pending());
}

#[test]
fn rescan_failure_after_recovery_reports_again() {
    let root = vault("rescan-again");
    let index = LazyIndex::new(root.clone());
    assert!(index.get().is_ok());
    fs::write(root.join("a.md"), [0xff, 0xfe, 0x00]).unwrap();
    assert_eq!(disk::apply(&index, &rescan()).errors.len(), 1);
    fs::write(root.join("a.md"), "# A\n").unwrap();
    assert!(disk::apply(&index, &rescan()).errors.is_empty());
    fs::write(root.join("a.md"), [0xff, 0xfe, 0x00]).unwrap();
    assert_eq!(disk::apply(&index, &rescan()).errors.len(), 1);
}

#[test]
fn recovery_clears_pending_retry() {
    let root = vault("rescan-clear");
    let index = LazyIndex::new(root.clone());
    assert!(index.get().is_ok());
    fs::write(root.join("a.md"), [0xff, 0xfe, 0x00]).unwrap();
    assert_eq!(disk::apply(&index, &rescan()).errors.len(), 1);
    fs::write(root.join("a.md"), "# A\n").unwrap();
    index.rebuild().unwrap();
    let mut batch = Batch::default();
    index
        .watch()
        .absorb(&mut batch, Change::Path(root.join("a.md")));
    assert!(!batch.rescan);
}

#[test]
fn busy_rescan_keeps_failure_report() {
    let root = vault("rescan-busy-report");
    let index = LazyIndex::new(root.clone());
    let held = index.get().unwrap();
    assert_eq!(disk::apply(&index, &rescan()).errors.len(), 1);
    drop(held);
    fs::write(root.join("a.md"), [0xff, 0xfe, 0x00]).unwrap();
    let batch = index.watch().drain();
    assert_eq!(disk::apply(&index, &batch).errors.len(), 1);
}

#[test]
fn rebuild_clears_deferred_retry() {
    let root = vault("rescan-rebuild");
    let index = LazyIndex::new(root);
    index.watch().rescan_later();
    index.rebuild().unwrap();
    assert!(!index.watch().pending());
}

#[test]
fn busy_names_do_not_consume_report() {
    let root = vault("rescan-busy-names");
    let index = LazyIndex::new(root);
    let held = index.names().unwrap();
    assert_eq!(disk::apply(&index, &rescan()).errors.len(), 1);
    assert_eq!(disk::apply(&index, &rescan()).errors.len(), 1);
    drop(held);
    assert!(index.watch().pending());
}
