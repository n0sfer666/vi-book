mod common;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use common::feed::{THREE, app, ctrl, line, map, map_command, press, text};
use v_ide_keys::MapMode;
use v_ide_text::Position;

#[test]
fn mutual_recursion_reports_e223() {
    let mut app = app(THREE);
    map(&mut app, MapMode::Normal, "a", "b", true);
    map(&mut app, MapMode::Normal, "b", "a", true);
    press(&mut app, "a");
    assert_eq!(app.ui.message, "E223: Recursive mapping");
}

#[test]
fn endless_expansion_stops_with_e223() {
    let mut app = app(THREE);
    map(&mut app, MapMode::Normal, "l", "ll", true);
    press(&mut app, "l");
    assert_eq!(app.ui.message, "E223: Recursive mapping");
    press(&mut app, "j");
    assert_eq!(line(&app), 1);
}

#[test]
fn failed_command_flushes_rest_of_rhs() {
    let mut app = app(THREE);
    map(&mut app, MapMode::Normal, "Q", ":nope<CR>dd", false);
    press(&mut app, "Qj");
    assert_eq!(text(&app), THREE);
    assert_eq!(line(&app), 1);
    app.workspace
        .current_mut()
        .editor
        .set_cursor(Position::new(0, 0));
    map(&mut app, MapMode::Normal, "W", ":help<CR>", false);
    press(&mut app, "W");
    assert!(app.ui.help);
}

#[test]
fn successful_command_keeps_rest_of_rhs() {
    let mut app = app(THREE);
    map(&mut app, MapMode::Normal, "W", ":help<CR>q", false);
    press(&mut app, "W");
    assert!(!app.ui.help);
}

#[test]
fn refused_quit_flushes_rest_of_rhs() {
    let mut app = app(THREE);
    map(&mut app, MapMode::Normal, "Q", ":q<CR>dd", false);
    press(&mut app, "xQ");
    assert!(!app.quit);
    assert_eq!(text(&app), "ne\ntwo\nthree");
}

#[test]
fn failed_action_inside_remap_flushes_rest_of_rhs() {
    let mut app = app(THREE);
    map_command(&mut app, "Q", "nope");
    map(&mut app, MapMode::Normal, "Z", "Qdd", true);
    press(&mut app, "Z");
    assert_eq!(text(&app), THREE);
}

#[test]
fn count_before_action_mapping_is_discarded() {
    let mut app = app(THREE);
    map_command(&mut app, "Q", "only");
    press(&mut app, "2Q");
    press(&mut app, "x");
    assert_eq!(text(&app), "ne\ntwo\nthree");
}

#[test]
fn count_before_colon_opens_command_line() {
    let mut app = app(THREE);
    press(&mut app, "2:");
    assert_eq!(app.ui.command.as_deref(), Some(""));
    press(&mut app, "help");
    app.handle(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert!(app.ui.help);
}

#[test]
fn count_before_colon_mapping_runs_command() {
    let mut app = app(THREE);
    map(&mut app, MapMode::Normal, "Q", ":help<CR>", false);
    press(&mut app, "2Q");
    assert!(app.ui.help);
}

#[test]
fn non_key_event_is_ignored() {
    let mut app = app(THREE);
    press(&mut app, ":nope");
    app.handle(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    let message = app.ui.message.clone();
    assert!(!message.is_empty());
    app.handle(KeyEvent::new(KeyCode::Null, KeyModifiers::NONE));
    assert_eq!(app.ui.message, message);
    ctrl(&mut app, 'w');
    app.handle(KeyEvent::new(KeyCode::Null, KeyModifiers::NONE));
    press(&mut app, "v");
    assert_eq!(app.pane_count(), 2);
}
