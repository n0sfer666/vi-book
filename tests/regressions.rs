mod common;

use std::fs;
use std::path::PathBuf;

use common::{check, editor_at, t};
use crossterm::event::{KeyCode, KeyEvent};
use v_ide_editor::document::Document;
use v_ide_engine::{Editor, Mode};
use v_ide_keys::{Key, parse_keys};
use v_ide_tui::actions;
use v_ide_tui::app::App;
use v_ide_tui::command_line;

#[test]
fn visual_operator_on_empty_buffer_is_safe() {
    for keys in ["vd", "vy", "vx"] {
        let mut editor = Editor::new("");
        editor.handle_keys(keys);
        assert_eq!(editor.text(), "", "keys {keys:?}");
        assert_eq!(editor.mode(), Mode::Normal, "keys {keys:?}");
    }
}

#[test]
fn change_to_line_end_on_empty_line_enters_insert() {
    let mut editor = editor_at("a\n\nb", 1, 0);
    editor.handle_keys("c$");
    assert_eq!(editor.mode(), Mode::Insert);
    editor.handle_keys("X<Esc>");
    assert_eq!(editor.text(), "a\nX\nb");
}

#[test]
fn linewise_operator_at_buffer_edge_is_noop() {
    check(&[
        t(
            "dj on last line does nothing",
            "a\nb",
            (1, 0),
            "dj",
            "a\nb",
            (1, 0),
        ),
        t(
            "dk on first line does nothing",
            "a\nb",
            (0, 0),
            "dk",
            "a\nb",
            (0, 0),
        ),
    ]);
}

#[test]
fn unknown_key_notation_is_skipped() {
    assert_eq!(parse_keys("<Wat>x"), vec![Key::Char('x')]);
}

fn temp_file(name: &str, content: &[u8]) -> PathBuf {
    let path = std::env::temp_dir().join(format!("vi-book-regr-{}-{name}", std::process::id()));
    fs::write(&path, content).unwrap();
    path
}

fn app_with_file(path: &PathBuf) -> App {
    let mut doc = Document::open(path).unwrap();
    doc.name = path.display().to_string();
    App::new(doc)
}

fn run_command(app: &mut App, command: &str) {
    app.command = Some(String::from(command));
    command_line::handle(app, KeyEvent::from(KeyCode::Enter));
}

#[test]
fn wq_saves_and_quits() {
    let path = temp_file("wq", b"hello\n");
    let mut app = app_with_file(&path);
    app.workspace.current_mut().editor.handle_keys("x");
    run_command(&mut app, "wq");
    let written = fs::read(&path).unwrap();
    fs::remove_file(&path).unwrap();
    assert!(app.quit, "wq must quit after a successful save");
    assert_eq!(written, b"ello\n");
}

#[test]
fn write_on_bundled_doc_fails_without_touching_disk() {
    let label = format!("vi-book-regr-{}-bundled.md", std::process::id());
    let mut app = App::new(common::named_doc("", ""));
    actions::open_doc(&mut app, &label, "bundled\n");
    run_command(&mut app, "w");
    assert!(!std::path::Path::new(&label).exists());
    assert_eq!(
        app.message,
        "save failed: buffer has no associated file path"
    );
}

#[test]
fn q_refuses_unsaved_changes_and_q_bang_discards() {
    let path = temp_file("q", b"hello\n");
    let mut app = app_with_file(&path);
    app.workspace.current_mut().editor.handle_keys("x");
    run_command(&mut app, "q");
    assert!(!app.quit, "q must refuse to quit with unsaved changes");
    run_command(&mut app, "q!");
    fs::remove_file(&path).unwrap();
    assert!(app.quit, "q! must discard changes and quit");
}

#[test]
fn colon_opens_command_line_only_when_idle() {
    let path = temp_file("colon", b"abc\n");
    let mut app = app_with_file(&path);
    fs::remove_file(&path).unwrap();
    app.handle(KeyEvent::from(KeyCode::Char(':')));
    assert_eq!(app.command.as_deref(), Some(""));
    app.command = None;
    app.workspace.current_mut().editor.handle_keys("d");
    app.handle(KeyEvent::from(KeyCode::Char(':')));
    assert!(
        app.command.is_none(),
        "colon must not interrupt a pending operator"
    );
}
