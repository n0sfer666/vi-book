use std::cell::{Cell, OnceCell, RefCell};
use std::io::{self, ErrorKind};
use std::mem;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

use crate::watcher::{Change, Watcher};

use super::batch::Batch;
use super::watch_state::State;

pub struct Watch {
    state: RefCell<State>,
    roots: OnceCell<(PathBuf, PathBuf)>,
    limited: Cell<bool>,
    deferred: Cell<bool>,
    stale: Cell<bool>,
    failing: Cell<bool>,
    vanished: Cell<bool>,
}

impl Watch {
    pub fn new() -> Self {
        Self {
            state: RefCell::new(State::Idle),
            roots: OnceCell::new(),
            limited: Cell::new(false),
            deferred: Cell::new(false),
            stale: Cell::new(false),
            failing: Cell::new(false),
            vanished: Cell::new(false),
        }
    }

    pub fn start(&self, root: &Path) {
        let Ok(mut state) = self.state.try_borrow_mut() else {
            return;
        };
        if !matches!(*state, State::Idle) {
            return;
        }
        let real = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
        let _ = self.roots.set((real, root.to_path_buf()));
        let (tx, rx) = mpsc::channel();
        let target = root.to_path_buf();
        let spawned = thread::Builder::new()
            .name(String::from("vault-watch"))
            .spawn(move || {
                let _ = tx.send(Watcher::new(&target));
            });
        *state = match spawned {
            Ok(_) => State::Starting(rx),
            Err(err) => State::Failed(Some(err.to_string())),
        };
    }

    pub fn rescan_later(&self) {
        self.deferred.set(true);
    }

    pub fn rescan_failed(&self, err: &io::Error) -> bool {
        if err.kind() != ErrorKind::NotFound {
            self.vanished.set(false);
        } else if !self.vanished.replace(true) {
            self.deferred.set(true);
            return false;
        }
        self.stale.set(true);
        !self.failing.replace(true)
    }

    #[cfg(test)]
    pub fn pending(&self) -> bool {
        self.deferred.get()
    }

    pub fn rescanned(&self) {
        self.deferred.set(false);
        self.stale.set(false);
        self.failing.set(false);
        self.vanished.set(false);
    }

    pub fn drain(&self) -> Batch {
        let mut batch = Batch {
            rescan: self.deferred.take(),
            ..Batch::default()
        };
        let Ok(mut state) = self.state.try_borrow_mut() else {
            return batch;
        };
        *state = match mem::replace(&mut *state, State::Idle) {
            State::Starting(rx) => match rx.try_recv() {
                Ok(Ok(watcher)) => State::Running(watcher),
                Ok(Err(err)) => State::Failed(Some(err.to_string())),
                Err(TryRecvError::Empty) => State::Starting(rx),
                Err(TryRecvError::Disconnected) => {
                    State::Failed(Some(String::from("watcher thread exited")))
                }
            },
            other => other,
        };
        match &mut *state {
            State::Failed(err) => batch.errors.extend(err.take()),
            State::Running(watcher) => {
                while let Some(change) = watcher.poll(Duration::ZERO) {
                    self.absorb(&mut batch, change);
                }
            }
            State::Idle | State::Starting(_) => {}
        }
        batch
    }

    pub(super) fn absorb(&self, batch: &mut Batch, change: Change) {
        match change {
            Change::Path(path) => {
                batch.paths.insert(self.rooted(path));
                batch.rescan |= self.stale.take();
            }
            Change::Rescan(err) => {
                batch.rescan = true;
                batch.errors.extend(err);
            }
            Change::Limit(err) => {
                if !self.limited.replace(true) {
                    batch.errors.push(err);
                }
            }
        }
    }

    fn rooted(&self, path: PathBuf) -> PathBuf {
        if let Some((real, root)) = self.roots.get()
            && let Ok(rest) = path.strip_prefix(real)
        {
            return root.join(rest);
        }
        path
    }
}
