use std::sync::mpsc::Receiver;

use crate::watcher::Watcher;

pub enum State {
    Idle,
    Starting(Receiver<notify::Result<Watcher>>),
    Running(Watcher),
    Failed(Option<String>),
}
