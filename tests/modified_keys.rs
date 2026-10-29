mod common;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use v_ide_tui::app::App;
use v_ide_tui::help;

fn press(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    app.handle(KeyEvent::new(code, modifiers));
}

fn plain(app: &mut App, code: KeyCode) {
    press(app, code, KeyModifiers::NONE);
}

fn text(app: &App) -> String {
    app.workspace.current().editor.text()
}

#[test]
fn modified_keys_in_insert_type_nothing() {
    let mut app = App::new(common::named_doc("ab", "a.md"));
    plain(&mut app, KeyCode::Char('i'));
    press(&mut app, KeyCode::Char('x'), KeyModifiers::ALT);
    press(&mut app, KeyCode::Enter, KeyModifiers::SHIFT);
    press(&mut app, KeyCode::Up, KeyModifiers::SHIFT);
    plain(&mut app, KeyCode::Char('y'));
    plain(&mut app, KeyCode::Esc);
    assert_eq!(text(&app), "yab");
}

#[test]
fn modified_arrows_in_normal_do_not_move() {
    let mut app = App::new(common::named_doc("ab\ncd", "a.md"));
    plain(&mut app, KeyCode::Char('j'));
    press(&mut app, KeyCode::Up, KeyModifiers::SHIFT);
    press(&mut app, KeyCode::Char('k'), KeyModifiers::ALT);
    assert_eq!(app.workspace.current().editor.cursor().line, 1);
}

#[test]
fn modified_keys_in_command_line_are_ignored() {
    let mut app = App::new(common::named_doc("ab", "a.md"));
    plain(&mut app, KeyCode::Char(':'));
    press(&mut app, KeyCode::Char('w'), KeyModifiers::CONTROL);
    press(&mut app, KeyCode::Char('x'), KeyModifiers::ALT);
    press(&mut app, KeyCode::Char('Q'), KeyModifiers::SHIFT);
    assert_eq!(app.ui.command.as_deref(), Some("Q"));
    press(&mut app, KeyCode::Enter, KeyModifiers::SHIFT);
    press(&mut app, KeyCode::Esc, KeyModifiers::ALT);
    assert_eq!(app.ui.command.as_deref(), Some("Q"));
}

#[test]
fn only_plain_f1_opens_help() {
    let mut app = App::new(common::named_doc("ab", "a.md"));
    press(&mut app, KeyCode::F(1), KeyModifiers::CONTROL);
    press(&mut app, KeyCode::F(1), KeyModifiers::SHIFT);
    assert!(!app.ui.help);
    plain(&mut app, KeyCode::F(1));
    assert!(app.ui.help);
    press(&mut app, KeyCode::Esc, KeyModifiers::ALT);
    assert!(app.ui.help);
    press(&mut app, KeyCode::F(1), KeyModifiers::CONTROL);
    assert!(app.ui.help);
    plain(&mut app, KeyCode::F(1));
    assert!(!app.ui.help);
}

#[test]
fn shift_enter_in_help_opens_the_developer_guide() {
    let mut app = App::new(common::named_doc("ab", "a.md"));
    plain(&mut app, KeyCode::F(1));
    press(&mut app, KeyCode::Enter, KeyModifiers::SHIFT);
    assert!(!app.ui.help);
    assert_eq!(text(&app), help::DEVELOPERS);
}

#[test]
fn modified_letters_do_not_reach_the_buffer_list_query() {
    let mut app = App::new(common::named_doc("ab", "a.md"));
    app.open_buffer_list();
    press(&mut app, KeyCode::Char('x'), KeyModifiers::ALT);
    press(&mut app, KeyCode::Char('u'), KeyModifiers::CONTROL);
    plain(&mut app, KeyCode::Char('a'));
    let query = app
        .ui
        .buffer_list
        .as_ref()
        .map(|list| list.query().to_string());
    assert_eq!(query.as_deref(), Some("a"));
}
