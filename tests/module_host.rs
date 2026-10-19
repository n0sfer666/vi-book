#[path = "module_host/fakes.rs"]
mod fakes;

use std::path::{Path, PathBuf};

use fakes::{Tally, all};
use v_ide_editor::document::Document;
use v_ide_editor::module::{Event, Request};
use v_ide_editor::workspace::Workspace;

fn ws() -> Workspace {
    let mut doc = Document::new("hello");
    doc.path = Some(PathBuf::from("n.md"));
    Workspace::new(doc)
}

fn msg(text: &str) -> Request {
    Request::Message(String::from(text))
}

#[test]
fn unknown_command_is_not_handled() {
    assert_eq!(all().command(&ws(), "nope", ""), None);
}

#[test]
fn first_command_activates_module_once() {
    let (mut modules, ws) = (all(), ws());
    assert_eq!(
        modules.command(&ws, "echo", "hi"),
        Some(vec![msg("activate"), msg("echo hi")])
    );
    assert_eq!(
        modules.command(&ws, "echo", "hi"),
        Some(vec![msg("echo hi")])
    );
}

#[test]
fn failed_activation_reports_on_each_command() {
    let (mut modules, ws) = (all(), ws());
    assert_eq!(modules.command(&ws, "broken", ""), Some(vec![msg("boom")]));
    assert_eq!(modules.command(&ws, "broken", ""), Some(vec![msg("boom")]));
}

#[test]
fn command_error_goes_after_queued_requests() {
    let (mut modules, ws) = (all(), ws());
    assert_eq!(
        modules.command(&ws, "faulty", ""),
        Some(vec![msg("partial"), msg("bad")])
    );
}

#[test]
fn events_reach_only_subscribers() {
    let (mut modules, ws) = (all(), ws());
    let open = Event::NoteOpen(PathBuf::from("n.md"));
    let save = Event::NoteSave(PathBuf::from("n.md"));
    assert_eq!(open.name(), "note_open");
    assert_eq!(save.name(), "note_save");
    modules.command(&ws, "echo", "");
    assert_eq!(
        modules.event(&ws, &open),
        vec![msg("listen"), msg("event note_open")]
    );
    assert_eq!(modules.event(&ws, &open), vec![msg("event note_open")]);
    assert_eq!(modules.event(&ws, &save), Vec::new());
}

#[test]
fn startup_activates_startup_modules() {
    let (mut modules, ws) = (all(), ws());
    assert_eq!(modules.startup(&ws), vec![msg("up")]);
    assert_eq!(modules.command(&ws, "boot", ""), Some(vec![msg("boot")]));
    assert_eq!(
        modules.command(&ws, "echo", "x"),
        Some(vec![msg("activate"), msg("echo x")])
    );
}

#[test]
fn requests_keep_call_order() {
    let (mut modules, ws) = (all(), ws());
    let expected = vec![
        Request::Open(PathBuf::from("a.md")),
        Request::Pick {
            items: vec![String::from("x")],
            command: String::from("choose"),
        },
        Request::List {
            title: String::from("Title"),
            rows: vec![String::from("r")],
            command: String::from("go"),
        },
        Request::Emit(Event::NoteSave(PathBuf::from("a.md"))),
        msg("Some(\"n.md\") hello"),
    ];
    assert_eq!(modules.command(&ws, "effects", ""), Some(expected));
}

#[test]
fn get_is_none_until_module_is_active() {
    let (mut modules, ws) = (all(), ws());
    assert!(modules.get::<Tally>().is_none());
    modules.command(&ws, "echo", "");
    assert!(modules.get::<Tally>().is_none());
    modules.command(&ws, "tally", "");
    assert_eq!(modules.get::<Tally>().map(|t| t.count), Some(1));
}

#[test]
fn get_mut_reaches_module_state() {
    let (mut modules, ws) = (all(), ws());
    assert!(modules.get_mut::<Tally>().is_none());
    modules.command(&ws, "tally", "");
    if let Some(tally) = modules.get_mut::<Tally>() {
        tally.count = 10;
    }
    assert_eq!(modules.command(&ws, "tally", ""), Some(vec![msg("11")]));
}

#[test]
fn cx_sees_workspace_cwd() {
    let (mut modules, mut ws) = (all(), ws());
    assert_eq!(ws.cwd(), Path::new("."));
    ws.set_cwd(PathBuf::from("/vault"));
    assert_eq!(ws.cwd(), Path::new("/vault"));
    assert_eq!(ws.absolute("a/b.md"), PathBuf::from("/vault/a/b.md"));
    assert_eq!(modules.command(&ws, "where", ""), Some(vec![msg("/vault")]));
}
