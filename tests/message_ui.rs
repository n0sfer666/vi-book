mod common;

use common::feed::{app, line, press};
use v_ide_editor::module::Request;
use v_ide_tui::{command_line::run, requests};

fn enter(app: &mut v_ide_tui::app::App) {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    app.handle(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
}

fn emit(app: &mut v_ide_tui::app::App, text: &str) {
    requests::apply(app, vec![Request::Message(text.into())]);
}

fn history(app: &mut v_ide_tui::app::App, count: usize) {
    for n in 0..count {
        emit(app, &format!("message {n}"));
    }
}

#[test]
fn output_covers_only_bottom_rows_and_places_cursor_on_prompt() {
    let mut app = app(&["background"; 12].join("\n"));
    emit(&mut app, "first\nsecond");
    let frame = common::draw(&mut app, 40, 10);
    assert!(frame.grid.row_text(6).starts_with("background"));
    assert!(frame.grid.row_text(7).starts_with("first"));
    assert!(frame.grid.row_text(8).starts_with("second"));
    assert!(frame.grid.row_text(9).starts_with("Press ENTER"));
    assert_eq!(frame.cursor.map(|(_, row)| row), Some(9));
    enter(&mut app);
    assert!(app.ui.pager.is_none());
    assert_eq!(line(&app), 0);
}

#[test]
fn pager_navigation_bypasses_mappings_and_hit_enter_replays_the_next_command() {
    let mut app = app("one\ntwo\nthree");
    run(&mut app, "nnoremap j G");
    common::draw(&mut app, 40, 7);
    history(&mut app, 20);
    run(&mut app, "messages");
    press(&mut app, "j");
    let frame = common::draw(&mut app, 40, 7);
    assert!(frame.grid.row_text(0).starts_with("message 1"));
    assert_eq!(line(&app), 0);
    press(&mut app, "q");
    emit(&mut app, "a\nb");
    press(&mut app, "j");
    assert!(app.ui.pager.is_none());
    assert_eq!(line(&app), 2);
}

#[test]
fn long_mapping_output_suspends_both_keys_and_actions_until_dismissed() {
    let mut app = app("one\ntwo\nthree");
    common::draw(&mut app, 40, 7);
    history(&mut app, 20);
    run(&mut app, "nnoremap Z <Cmd>messages<CR>j<Cmd>q!<CR>");
    press(&mut app, "Z");
    assert!(app.ui.pager.as_ref().unwrap().more());
    assert_eq!(line(&app), 0);
    assert!(!app.quit);
    press(&mut app, "q");
    assert!(app.ui.pager.is_none());
    assert_eq!(line(&app), 1);
    assert!(app.quit);
}

#[test]
fn small_mapping_output_does_not_interrupt_the_remaining_mapping() {
    let mut app = app("one\ntwo\nthree");
    history(&mut app, 2);
    run(&mut app, "nnoremap Z <Cmd>messages<CR>j");
    press(&mut app, "Z");
    assert!(app.ui.pager.is_none());
    assert_eq!(line(&app), 1);
}

#[test]
fn colon_starts_command_line_and_messages_do_not_interrupt_submission() {
    let mut app = app("one\ntwo");
    emit(&mut app, "a\nb");
    press(&mut app, ":only");
    assert!(app.ui.pager.is_none());
    assert_eq!(app.ui.command.as_deref(), Some("only"));
    emit(&mut app, "c\nd");
    assert!(app.ui.pager.is_none());
    enter(&mut app);
    assert!(app.ui.pager.is_none());
    assert!(app.ui.command.is_none());
}
