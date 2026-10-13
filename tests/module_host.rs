#[path = "module_host/fakes.rs"]
mod fakes;

use std::path::PathBuf;

use fakes::all;
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
fn failed_activation_reports_and_retries() {
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
