mod common;

use std::fs;
use v_ide_editor::document::Document;
use v_ide_tui::{app::App, command_line};

#[test]
fn readonly_blocks_write_and_wq_but_successful_force_clears_only_local_flag() {
    let root = common::scratch("readonly-writes");
    let path = root.join("a.txt");
    fs::write(&path, "one").unwrap();
    let mut app = App::new(Document::open(&path).unwrap());
    app.workspace.current_mut().editor.handle_keys("Ax<Esc>");
    command_line::run(&mut app, "set ro");
    for command in ["w", "wq"] {
        command_line::run(&mut app, command);
        assert!(app.ui.message.contains("E45"));
        assert!(!app.quit);
        assert!(app.workspace.current().is_dirty());
        assert_eq!(fs::read_to_string(&path).unwrap(), "one");
    }
    command_line::run(&mut app, "w!");
    assert_eq!(fs::read_to_string(&path).unwrap(), "onex");
    assert!(!app.workspace.current().is_dirty());
    command_line::run(&mut app, "set ro?");
    assert_eq!(app.ui.message, "noreadonly");
    command_line::run(&mut app, "setglobal ro?");
    assert_eq!(app.ui.message, "  readonly");
}

#[test]
fn failed_force_clears_local_readonly_even_when_the_file_cannot_be_written() {
    let root = common::scratch("readonly-failed-force");
    let mut doc = Document::new("draft");
    doc.path = Some(root.join("absent").join("a.txt"));
    let mut app = App::new(doc);
    command_line::run(&mut app, "setlocal ro");
    command_line::run(&mut app, "w!");
    assert!(app.ui.message.contains("save failed"));
    command_line::run(&mut app, "set ro?");
    assert_eq!(app.ui.message, "noreadonly");
}
