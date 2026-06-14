use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::style::Modifier;

use vi_book::Editor;
use vi_book::ui::app::App;
use vi_book::ui::view::View;
use vi_book::vim::mode::Mode;

fn ctrl(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
}

fn plain(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
}

#[test]
fn visual_selection_is_highlighted() {
    let mut app = App::new(Editor::new("hello world"), "a.md".into());
    app.view = View::Edit;
    app.editor.handle_keys("vll");
    let backend = TestBackend::new(100, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| vi_book::ui::render::draw(frame, &mut app))
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    // content is centred in a 90-col column: left margin = (100-90)/2 = 5
    let reversed = buf[(5, 0)]
        .style()
        .add_modifier
        .contains(Modifier::REVERSED);
    assert!(reversed, "selected cell should be reversed");
    let plain_cell = buf[(20, 0)]
        .style()
        .add_modifier
        .contains(Modifier::REVERSED);
    assert!(!plain_cell, "unselected cell should not be reversed");
}

#[test]
fn read_mode_delegates_v_to_visual() {
    let mut app = App::new(Editor::new("hello"), "a.md".into());
    app.view = View::Read;
    app.handle(plain('v'));
    assert_eq!(app.view, View::Edit);
    assert_eq!(app.editor.mode(), Mode::Visual);
}

#[test]
fn read_mode_delegates_ctrl_v_to_visual_block() {
    let mut app = App::new(Editor::new("abc\ndef"), "a.md".into());
    app.view = View::Read;
    app.handle(ctrl('v'));
    assert_eq!(app.view, View::Edit);
    assert_eq!(app.editor.mode(), Mode::VisualBlock);
}

#[test]
fn read_mode_motions_still_work() {
    let mut app = App::new(Editor::new("# Title\n\nbody"), "a.md".into());
    app.view = View::Read;
    app.handle(plain('j'));
    assert_eq!(app.view, View::Read, "j stays in read mode");
}
