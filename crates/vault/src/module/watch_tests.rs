use std::collections::BTreeSet;
use std::fs;
use std::io::{self, ErrorKind};

use crate::watcher::Change;

use super::batch::Batch;
use super::disk;
use super::lazy::LazyIndex;
use super::plan::Plan;
use super::test_vault::vault;
use super::watch::Watch;

#[test]
fn known_paths_under_dir_are_not_reread() {
    let root = vault("plan-known");
    fs::write(root.join("sub/c.md"), "# C\n").unwrap();
    let paths = BTreeSet::from([root.join("sub")]);
    let plan = Plan::new(&root, false, &paths, &[String::from("sub/b.md")]);
    assert_eq!(
        plan.found.keys().cloned().collect::<Vec<_>>(),
        vec![String::from("sub/c.md")]
    );
    assert!(plan.gone.is_empty());
}

#[test]
fn limit_is_reported_once() {
    let watch = Watch::new();
    let mut batch = Batch::default();
    watch.absorb(&mut batch, Change::Limit(String::from("limit")));
    watch.absorb(&mut batch, Change::Limit(String::from("limit")));
    assert_eq!(batch.errors, vec![String::from("limit")]);
    assert!(!batch.rescan);
}

#[test]
fn busy_index_defers_rescan() {
    let root = vault("busy");
    let index = LazyIndex::new(root.clone());
    let held = index.get().unwrap();
    let batch = Batch {
        paths: BTreeSet::from([root.join("a.md")]),
        ..Batch::default()
    };
    let applied = disk::apply(&index, &batch);
    drop(held);
    assert!(!applied.changed);
    assert_eq!(applied.errors.len(), 1);
    assert!(index.watch().drain().rescan);
}

#[test]
fn other_failure_breaks_vanished_streak() {
    let watch = Watch::new();
    let vanished = io::Error::from(ErrorKind::NotFound);
    assert!(!watch.rescan_failed(&vanished));
    assert!(watch.drain().rescan);
    assert!(watch.rescan_failed(&io::Error::from(ErrorKind::InvalidData)));
    assert!(!watch.rescan_failed(&vanished));
    assert!(watch.pending());
}
