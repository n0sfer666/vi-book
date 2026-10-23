use std::path::PathBuf;

use notify::{ErrorKind, Event, EventKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    Path(PathBuf),
    Rescan(Option<String>),
    Limit(String),
}

pub fn changes(res: notify::Result<Event>) -> Vec<Change> {
    match res {
        Err(err) if matches!(err.kind, ErrorKind::MaxFilesWatch) => {
            vec![Change::Limit(err.to_string())]
        }
        Err(err) => vec![Change::Rescan(Some(err.to_string()))],
        Ok(event) if event.need_rescan() => vec![Change::Rescan(None)],
        Ok(event) if is_change(&event.kind) => event.paths.into_iter().map(Change::Path).collect(),
        Ok(_) => Vec::new(),
    }
}

fn is_change(kind: &EventKind) -> bool {
    matches!(
        kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    )
}
