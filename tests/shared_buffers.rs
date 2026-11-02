mod common;

use std::fs;
use std::path::PathBuf;

use v_ide_editor::document::Document;
use v_ide_text::Position;
use v_ide_tui::{app::App, command_line, requests};

fn file_app(label: &str) -> (App, PathBuf) {
    let root = common::scratch(label);
    fs::write(root.join("a.txt"), "alpha\nbeta\ngamma").unwrap();
    fs::write(root.join("b.txt"), "other").unwrap();
    let mut doc = Document::open(root.join("a.txt")).unwrap();
    doc.name = "a.txt".into();
    let mut app = App::new(doc);
    app.workspace.set_cwd(root.clone());
    (app, root)
}

#[test]
fn shared_text_dirty_state_and_undo_belong_to_one_buffer() {
    let mut app = App::new(common::named_doc("alpha", "a.md"));
    let id = app.workspace.current_id();
    app.vsplit(None);
    assert_eq!(app.workspace.current_id(), id);
    app.workspace.current_mut().editor.handle_keys("x");
    app.focus_next();
    assert_eq!(app.workspace.current().editor.text(), "lpha");
    assert!(app.workspace.current().is_dirty());
    app.workspace.current_mut().editor.handle_keys("u");
    app.focus_next();
    assert_eq!(app.workspace.current().editor.text(), "alpha");
    assert!(!app.workspace.current().is_dirty());
}

#[test]
fn split_copies_the_view_but_keeps_cursor_and_desired_column_independent() {
    let mut app = App::new(common::named_doc("abcdef\nxy\nabcdefghijkl", "a.md"));
    app.workspace.current_mut().editor.handle_keys("$");
    app.vsplit(None);
    assert_eq!(app.workspace.current().editor.cursor(), Position::new(0, 5));
    app.workspace.current_mut().editor.handle_keys("0j");
    app.focus_next();
    assert_eq!(app.workspace.current().editor.cursor(), Position::new(0, 5));
    app.workspace.current_mut().editor.handle_keys("jj");
    assert_eq!(
        app.workspace.current().editor.cursor(),
        Position::new(2, 11)
    );
    app.focus_next();
    assert_eq!(app.workspace.current().editor.cursor(), Position::new(1, 0));
}

#[test]
fn buffer_switch_restores_the_view_for_each_window_and_reuses_visible_buffers() {
    let (mut app, _) = file_app("shared-window-history");
    let id = app.workspace.current_id();
    app.workspace
        .current_mut()
        .editor
        .set_cursor(Position::new(0, 4));
    app.vsplit(None);
    app.workspace
        .current_mut()
        .editor
        .set_cursor(Position::new(1, 2));
    app.ui.read.top = 1;
    app.open_rel("b.txt");
    app.open_rel("a.txt");
    assert_eq!(app.workspace.current_id(), id);
    assert_eq!(app.workspace.current().editor.cursor(), Position::new(1, 2));
    assert_eq!(app.ui.read.top, 1);
    assert!(app.workspace.pool().iter().all(|buffer| buffer.id != id));
    app.focus_next();
    assert_eq!(app.workspace.current().editor.cursor(), Position::new(0, 4));
    assert_eq!(app.ui.read.top, 0);
}

#[test]
fn explicit_split_target_reuses_dirty_resident_text_without_reading_it_again() {
    let (mut app, root) = file_app("shared-explicit-split");
    let id = app.workspace.current_id();
    app.workspace.current_mut().editor.handle_keys("Ax<Esc>");
    app.vsplit(Some("a.txt"));
    assert_eq!(app.workspace.current_id(), id);
    assert_eq!(app.workspace.current().editor.text(), "alphax\nbeta\ngamma");
    assert_eq!(
        fs::read_to_string(root.join("a.txt")).unwrap(),
        "alpha\nbeta\ngamma"
    );
    command_line::run(&mut app, "w");
    app.focus_next();
    assert!(!app.workspace.current().is_dirty());
}

#[test]
fn replacing_one_unnamed_window_does_not_drop_the_other_windows_buffer() {
    let (mut app, _) = file_app("shared-unnamed-retention");
    let _ = app.workspace.replace(Document::new("draft"));
    let id = app.workspace.current_id();
    app.vsplit(None);
    app.open_rel("b.txt");
    app.focus_next();
    assert_eq!(app.workspace.current_id(), id);
    assert_eq!(app.workspace.current().editor.text(), "draft");
}

#[test]
fn external_reload_is_checked_once_and_shared_by_all_windows() {
    let (mut app, root) = file_app("shared-single-reload");
    app.vsplit(None);
    fs::write(root.join("a.txt"), "external").unwrap();
    assert!(requests::tick(&mut app));
    assert!(app.ui.message.contains("reloaded"));
    assert!(!app.ui.message.contains("more"));
    app.focus_next();
    assert_eq!(app.workspace.current().editor.text(), "external");
    app.workspace.current_mut().editor.handle_keys("u");
    app.focus_next();
    assert_eq!(app.workspace.current().editor.text(), "alpha\nbeta\ngamma");
}

#[test]
fn selecting_from_an_empty_initial_buffer_initializes_the_target_view() {
    let mut app = App::new(Document::new(""));
    app.ui.read.top = 7;
    let mut doc = Document::new("target");
    doc.name = "b.txt".into();
    app.workspace.add(doc);
    app.switch_to_buffer(0);
    assert_eq!(app.workspace.current().editor.text(), "target");
    assert_eq!(app.ui.read.top, 0);
    assert_eq!(app.ui.read.view, v_ide_tui::view::View::Edit);
}
