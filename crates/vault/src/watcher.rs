use std::path::Path;
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

use notify::{Event, RecursiveMode, Watcher as _};

pub use crate::change::{Change, changes};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiskChange {
    Modified,
    Removed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reaction {
    Reload,
    Conflict,
    Vanished,
    Ignore,
}

pub fn react(buffer_dirty: bool, change: DiskChange) -> Reaction {
    match (change, buffer_dirty) {
        (DiskChange::Removed, _) => Reaction::Vanished,
        (DiskChange::Modified, false) => Reaction::Reload,
        (DiskChange::Modified, true) => Reaction::Conflict,
    }
}

pub struct Watcher {
    _inner: notify::RecommendedWatcher,
    rx: Receiver<Change>,
}

impl Watcher {
    pub fn new(root: &Path) -> notify::Result<Self> {
        let (tx, rx) = channel();
        let mut inner = notify::recommended_watcher(move |res: notify::Result<Event>| {
            for change in changes(res) {
                let _ = tx.send(change);
            }
        })?;
        inner.watch(root, RecursiveMode::Recursive)?;
        Ok(Self { _inner: inner, rx })
    }

    pub fn poll(&self, timeout: Duration) -> Option<Change> {
        self.rx.recv_timeout(timeout).ok()
    }
}
