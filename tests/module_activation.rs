#[path = "module_activation/fakes.rs"]
mod fakes;

use std::path::PathBuf;
use std::sync::atomic::Ordering;

use fakes::{ATTEMPTS, modules};
use v_ide_editor::document::Document;
use v_ide_editor::module::{Event, Request};
use v_ide_editor::workspace::Workspace;

fn ws() -> Workspace {
    Workspace::new(Document::new(""))
}

fn msg(text: &str) -> Request {
    Request::Message(String::from(text))
}

fn vault(notes: &[&str]) -> Event {
    Event::VaultChanged {
        notes: notes.iter().map(|n| n.to_string()).collect(),
        tags: Vec::new(),
        backlinks: Default::default(),
    }
}

fn open() -> Event {
    Event::NoteOpen(PathBuf::from("n.md"))
}

#[test]
fn failed_activation_is_remembered() {
    let (mut modules, ws) = (modules(), ws());
    assert_eq!(modules.command(&ws, "flaky", ""), Some(vec![msg("boom 1")]));
    assert_eq!(modules.command(&ws, "flaky", ""), Some(vec![msg("boom 1")]));
    assert_eq!(
        modules.event(&ws, &Event::NoteSave(PathBuf::from("n.md"))),
        Vec::new()
    );
    assert_eq!(ATTEMPTS.load(Ordering::SeqCst), 1);
}

#[test]
fn observe_does_not_activate() {
    let (mut modules, ws) = (modules(), ws());
    assert_eq!(modules.event(&ws, &open()), Vec::new());
    assert_eq!(modules.event(&ws, &vault(&["a.md"])), Vec::new());
}

#[test]
fn observe_reaches_active_module() {
    let (mut modules, ws) = (modules(), ws());
    assert_eq!(
        modules.command(&ws, "watch", ""),
        Some(vec![msg("watch up"), msg("watch")])
    );
    assert_eq!(modules.event(&ws, &open()), vec![msg("seen note_open")]);
}

#[test]
fn activation_replays_last_observed_events() {
    let (mut modules, ws) = (modules(), ws());
    modules.event(&ws, &open());
    modules.event(&ws, &vault(&["a.md"]));
    modules.event(&ws, &vault(&["a.md", "b.md"]));
    assert_eq!(
        modules.command(&ws, "watch", ""),
        Some(vec![
            msg("watch up"),
            msg("vault a.md,b.md"),
            msg("seen note_open"),
            msg("watch"),
        ])
    );
}

#[test]
fn activate_by_name_runs_once() {
    let (mut modules, ws) = (modules(), ws());
    modules.event(&ws, &vault(&["a.md"]));
    assert_eq!(
        modules.activate(&ws, "watcher"),
        vec![msg("watch up"), msg("vault a.md")]
    );
    assert_eq!(modules.activate(&ws, "watcher"), Vec::new());
    assert_eq!(modules.activate(&ws, "missing"), Vec::new());
}

#[test]
fn filetype_activates_matching_modules_once() {
    let (mut modules, ws) = (modules(), ws());
    assert_eq!(modules.filetype(&ws, "rust"), Vec::new());
    assert_eq!(modules.filetype(&ws, "markdown"), vec![msg("md up")]);
    assert_eq!(modules.filetype(&ws, "markdown"), Vec::new());
}
