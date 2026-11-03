use std::path::PathBuf;

use crate::module::Event;
use crate::options::{Access, Change};
use crate::workspace::{DocId, WindowId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    Open(PathBuf),
    Execute(super::Action),
    Message(String),
    Pick {
        items: Vec<String>,
        command: String,
    },
    List {
        title: String,
        rows: Vec<String>,
        command: String,
    },
    Emit(Event),
    SetText(String),
    SetOption {
        doc: DocId,
        window: WindowId,
        access: Access,
        change: Change,
    },
    OpenExternal(String),
    Command(String),
}
