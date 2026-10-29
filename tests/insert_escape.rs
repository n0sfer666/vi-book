mod common;

use std::time::Instant;

use common::feed::{app, press, text};
use v_ide_engine::mode::Mode;
use v_ide_keys::TIMEOUT;
use v_ide_tui::app::App;

fn mode(app: &App) -> Mode {
    app.workspace.current().editor.mode()
}

fn lua(label: &str, script: &str) -> App {
    common::lua_app(common::named_doc("", "a.md"), label, script)
}

fn rows(app: &mut App) -> Vec<String> {
    let grid = common::draw(app, 40, 8).grid;
    (0..8).map(|y| grid.row_text(y)).collect()
}

#[test]
fn jj_leaves_insert_without_inserting() {
    let mut app = app("");
    press(&mut app, "ihellojj");
    assert_eq!(text(&app), "hello");
    assert_eq!(mode(&app), Mode::Normal);
}

#[test]
fn single_j_followed_by_other_key_is_inserted() {
    let mut app = app("");
    press(&mut app, "ijazz");
    assert_eq!(text(&app), "jazz");
    assert_eq!(mode(&app), Mode::Insert);
}

#[test]
fn timeout_inserts_pending_j() {
    let mut app = app("");
    press(&mut app, "ij");
    app.timeout(Instant::now() + TIMEOUT);
    assert_eq!(text(&app), "j");
    assert_eq!(mode(&app), Mode::Insert);
}

#[test]
fn pending_j_is_drawn_at_cursor_but_not_in_buffer() {
    let mut app = app("");
    press(&mut app, "iabj");
    assert_eq!(text(&app), "ab");
    assert!(rows(&mut app).iter().any(|row| row.contains("abj")));
}

#[test]
fn insert_session_ended_by_jj_is_one_undo_step() {
    let mut app = app("x");
    press(&mut app, "ihellojju");
    assert_eq!(text(&app), "x");
}

#[test]
fn lua_insert_escape_sets_the_sequence() {
    let mut app = lua("esc-jk", "vibook.opt.insert_escape = 'jk'");
    press(&mut app, "ijjjk");
    assert_eq!(text(&app), "jj");
    assert_eq!(mode(&app), Mode::Normal);
}

#[test]
fn empty_insert_escape_disables_it() {
    let mut app = lua("esc-off", "vibook.opt.insert_escape = ''");
    press(&mut app, "ijj");
    assert_eq!(text(&app), "jj");
    assert_eq!(mode(&app), Mode::Insert);
}

#[test]
fn explicit_insert_keymap_beats_insert_escape() {
    let mut app = lua(
        "esc-clash",
        "vibook.opt.insert_escape = 'jj'\nvibook.keymap('i', 'jj', 'X')",
    );
    press(&mut app, "ijj");
    assert_eq!(text(&app), "X");
    assert_eq!(mode(&app), Mode::Insert);
}
