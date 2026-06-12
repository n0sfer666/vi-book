use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

use notify::{Event, RecursiveMode, Watcher as _};

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
    rx: Receiver<PathBuf>,
}

impl Watcher {
    pub fn new(root: &Path) -> notify::Result<Self> {
        let (tx, rx) = channel();
        let mut inner = notify::recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(event) = res
                && is_change(&event.kind)
            {
                for path in event.paths {
                    let _ = tx.send(path);
                }
            }
        })?;
        inner.watch(root, RecursiveMode::Recursive)?;
        Ok(Self { _inner: inner, rx })
    }

    pub fn poll(&self, timeout: Duration) -> Option<PathBuf> {
        self.rx.recv_timeout(timeout).ok()
    }
}

fn is_change(kind: &notify::EventKind) -> bool {
    use notify::EventKind;
    matches!(
        kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    )
}
