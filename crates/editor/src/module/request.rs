use std::path::PathBuf;

use crate::module::Event;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    Open(PathBuf),
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
}
