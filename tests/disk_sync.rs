mod common;

use std::fs;
use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use v_ide_editor::document::Document;
use v_ide_tui::app::App;
use v_ide_tui::view::View;
use v_ide_tui::{command_line, requests};

const CONFLICT: &str = "changed on disk (:e! load, :w! overwrite)";

fn vault(label: &str) -> (App, PathBuf) {
    let root = common::scratch(&format!("sync-{label}"));
    fs::write(root.join("a.md"), "alpha").unwrap();
    fs::write(root.join("b.md"), "beta").unwrap();
    let mut app = App::new(common::named_doc("", ""));
    common::open_vault(&mut app, &root);
    app.open_rel("a.md");
    (app, root)
}

fn twin(app: &App) -> &Document {
    let active = app.workspace.active_window();
    let window = app.workspace.windows().into_iter().find(|w| w.id != active);
    window.unwrap().buf.doc
}

fn status(app: &mut App) -> String {
    let grid = common::draw(app, 80, 24).grid;
    let y = grid.h - 2;
    (0..grid.w)
        .map(|x| grid.cell(x, y).symbol.as_str().to_string())
        .collect()
}

#[test]
fn write_syncs_clean_split_of_same_file_quietly() {
    let (mut app, _root) = vault("twin-clean");
    app.vsplit(None);
    app.workspace.current_mut().editor.handle_keys("Amine<Esc>");
    command_line::run(&mut app, "w");
    assert_eq!(twin(&app).editor.text(), "alphamine");
    assert!(!twin(&app).is_dirty());
    assert!(!requests::tick(&mut app));
    assert_eq!(app.ui.message, "written");
}

#[test]
fn write_clears_shared_dirty_state_in_all_windows() {
    let (mut app, _root) = vault("twin-dirty");
    app.vsplit(None);
    app.workspace.current_mut().editor.handle_keys("Aleft<Esc>");
    app.focus_next();
    app.workspace
        .current_mut()
        .editor
        .handle_keys("Aright<Esc>");
    command_line::run(&mut app, "w");
    assert_eq!(twin(&app).editor.text(), "alphaleftright");
    assert_eq!(twin(&app).disk_mark(), None);
    requests::tick(&mut app);
    assert_eq!(app.ui.message, "written");
}

#[test]
fn write_rechecks_disk_before_the_tick_does() {
    let (mut app, root) = vault("recheck");
    app.workspace.current_mut().editor.handle_keys("Amine<Esc>");
    fs::write(root.join("a.md"), "theirs").unwrap();
    command_line::run(&mut app, "w");
    assert_eq!(fs::read_to_string(root.join("a.md")).unwrap(), "theirs");
    assert_eq!(app.ui.message, format!("\"a.md\" {CONFLICT}"));
    assert!(status(&mut app).contains("[changed]"));
}

#[test]
fn conflict_outranks_reload_in_one_tick() {
    let (mut app, root) = vault("rank-conflict");
    app.open_rel("b.md");
    app.workspace.current_mut().editor.handle_keys("Amine<Esc>");
    fs::write(root.join("a.md"), "alpha two").unwrap();
    fs::write(root.join("b.md"), "theirs").unwrap();
    requests::tick(&mut app);
    assert_eq!(app.ui.message, format!("\"b.md\" {CONFLICT} (+1 more)"));
}

#[test]
fn deletion_outranks_reload_in_one_tick() {
    let (mut app, root) = vault("rank-deleted");
    app.open_rel("b.md");
    fs::write(root.join("b.md"), "beta two").unwrap();
    fs::remove_file(root.join("a.md")).unwrap();
    requests::tick(&mut app);
    assert_eq!(app.ui.message, "\"a.md\" deleted on disk (+1 more)");
}

#[test]
fn clearing_a_mark_on_tick_redraws() {
    let (mut app, root) = vault("unmark");
    app.workspace.current_mut().editor.handle_keys("Amine<Esc>");
    fs::write(root.join("a.md"), "theirs").unwrap();
    requests::tick(&mut app);
    fs::write(root.join("a.md"), "alpha").unwrap();
    assert!(requests::tick(&mut app));
    assert!(!status(&mut app).contains("[changed]"));
}

#[test]
fn edit_bang_marks_vanished_file_before_the_tick_does() {
    let (mut app, root) = vault("bang-gone");
    fs::remove_file(root.join("a.md")).unwrap();
    command_line::run(&mut app, "e!");
    assert_eq!(app.ui.message, "\"a.md\" deleted on disk");
    assert!(status(&mut app).contains("a.md [deleted]"));
    app.ui.message.clear();
    assert!(!requests::tick(&mut app));
    assert_eq!(app.ui.message, "");
}

#[test]
fn edit_bang_on_file_never_written_says_not_on_disk() {
    let root = common::scratch("sync-bang-never");
    let mut doc = common::named_doc("draft", "ghost.md");
    doc.path = Some(root.join("ghost.md"));
    let mut app = App::new(doc);
    command_line::run(&mut app, "e!");
    assert_eq!(app.ui.message, "\"ghost.md\" not on disk");
    assert!(!status(&mut app).contains("[deleted]"));
}

#[test]
fn read_selection_holds_reload_of_current_buffer() {
    let (mut app, root) = vault("read-select");
    app.ui.read.view = View::Read;
    app.handle(KeyEvent::new(KeyCode::Char('v'), KeyModifiers::NONE));
    fs::write(root.join("a.md"), "alpha two").unwrap();
    requests::tick(&mut app);
    assert_eq!(app.workspace.current().editor.text(), "alpha");
    app.handle(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    requests::tick(&mut app);
    assert_eq!(app.workspace.current().editor.text(), "alpha two");
}
