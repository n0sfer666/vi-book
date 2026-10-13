mod common;

use std::fs;
use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use v_ide_markdown::render::render;
use v_ide_tui::actions;
use v_ide_tui::app::App;
use v_ide_tui::help;
use v_ide_tui::read_motion;
use v_ide_tui::session::Session;
use v_ide_tui::view::View;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn temp_vault(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("vi-book-ui-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

#[test]
fn note_opens_in_read_mode_by_default() {
    let root = temp_vault("read");
    fs::write(root.join("a.md"), "# Title\n\nbody").unwrap();
    let (session, _) = Session::open(Some(root.to_str().unwrap())).unwrap();
    let mut app = App::new(common::named_doc("", ""));
    app.session = session;
    app.load("a.md");
    let view = app.ui.read.view;
    let _ = fs::remove_dir_all(&root);
    assert_eq!(view, View::Read);
}

#[test]
fn leader_opens_the_menu_in_normal_mode() {
    let mut app = App::new(common::named_doc("hi", "a.md"));
    assert!(app.ui.menu.is_none());
    app.handle(key(KeyCode::Char(' ')));
    assert!(app.ui.menu.is_some());
}

#[test]
fn menu_new_entry_prompts_for_a_name() {
    let mut app = App::new(common::named_doc("hi", "a.md"));
    app.handle(key(KeyCode::Char(' ')));
    app.handle(key(KeyCode::Char('n')));
    assert_eq!(app.ui.command.as_deref(), Some("new "));
}

#[test]
fn f1_opens_help_and_enter_reveals_the_manual() {
    let mut app = App::new(common::named_doc("hi", "a.md"));
    app.handle(key(KeyCode::F(1)));
    assert!(app.ui.help);
    actions::help_key(&mut app, key(KeyCode::Enter));
    assert!(!app.ui.help);
    assert_eq!(app.workspace.current().editor.text(), help::MANUAL);
    assert_eq!(app.ui.read.view, View::Read);
}

#[test]
fn empty_vault_is_detected() {
    let root = temp_vault("empty");
    let (session, _) = Session::open(Some(root.to_str().unwrap())).unwrap();
    let empty = session.is_empty();
    let _ = fs::remove_dir_all(&root);
    assert!(empty);
}

#[test]
fn read_word_motion_advances_within_a_line() {
    let rendered = render("alpha beta gamma", 80);
    assert_eq!(read_motion::word_forward(&rendered, (0, 0)), (0, 6));
}

#[test]
fn read_element_motion_jumps_blocks() {
    let rendered = render("# A\n\nbody text\n\n# B", 80);
    let forward = read_motion::element_forward(&rendered, (0, 0));
    assert!(forward.0 > 0, "element forward should leave the first line");
    let back = read_motion::element_back(&rendered, forward);
    assert_eq!(back.0, 0);
}
