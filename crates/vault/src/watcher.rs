use std::path::Path;
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

use notify::{Event, RecursiveMode, Watcher as _};

pub use crate::change::{Change, changes};

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
