use std::path::PathBuf;

use notify::event::{AccessKind, CreateKind, Flag};
use notify::{Event, EventKind};
use v_ide_vault::watcher::{Change, changes};

#[test]
fn change_event_yields_its_paths() {
    let event = Event::new(EventKind::Create(CreateKind::File))
        .add_path(PathBuf::from("/v/a.md"))
        .add_path(PathBuf::from("/v/b.md"));
    assert_eq!(
        changes(Ok(event)),
        vec![
            Change::Path(PathBuf::from("/v/a.md")),
            Change::Path(PathBuf::from("/v/b.md")),
        ]
    );
}

#[test]
fn rescan_flag_yields_rescan() {
    let event = Event::new(EventKind::Other).set_flag(Flag::Rescan);
    assert_eq!(changes(Ok(event)), vec![Change::Rescan(None)]);
}

#[test]
fn notify_error_yields_rescan_with_message() {
    assert_eq!(
        changes(Err(notify::Error::generic("boom"))),
        vec![Change::Rescan(Some(String::from("boom")))]
    );
}

#[test]
fn access_event_is_ignored() {
    let event = Event::new(EventKind::Access(AccessKind::Read)).add_path(PathBuf::from("/v/a.md"));
    assert_eq!(changes(Ok(event)), Vec::new());
}

#[test]
fn watch_limit_yields_limit() {
    let err = notify::Error::new(notify::ErrorKind::MaxFilesWatch);
    assert_eq!(
        changes(Err(err)),
        vec![Change::Limit(String::from("OS file watch limit reached."))]
    );
}
