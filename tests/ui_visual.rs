use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::backend::TestBackend;
use ratatui::style::Modifier;
use ratatui::Terminal;

use vi_book::ui::app::App;
use vi_book::ui::view::View;
use vi_book::vim::mode::Mode;
use vi_book::Editor;

fn ctrl(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
}

fn plain(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
}

fn reversed_at(app: &mut App, x: u16, y: u16) -> bool {
    let backend = TestBackend::new(100, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| vi_book::ui::render::draw(frame, app))
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    buf[(x, y)]
        .style()
        .add_modifier
        .contains(Modifier::REVERSED)
}

#[test]
fn visual_selection_is_highlighted_in_edit() {
    let mut app = App::new(Editor::new("hello world"), "a.md".into());
    app.view = View::Edit;
    app.editor.handle_keys("vll");
    assert!(
        reversed_at(&mut app, 0, 0),
        "selected cell should be reversed"
    );
}

#[test]
fn read_mode_v_selects_without_leaving_read() {
    let mut app = App::new(Editor::new("hello"), "a.md".into());
    app.view = View::Read;
    app.handle(plain('v'));
    assert_eq!(app.view, View::Read, "selection stays in read mode");
    assert_eq!(app.read_select.map(|(mode, _)| mode), Some(Mode::Visual));
}

#[test]
fn read_mode_ctrl_v_selects_block_in_read() {
    let mut app = App::new(Editor::new("abc\ndef"), "a.md".into());
    app.view = View::Read;
    app.handle(ctrl('v'));
    assert_eq!(app.view, View::Read);
    assert_eq!(
        app.read_select.map(|(mode, _)| mode),
        Some(Mode::VisualBlock)
    );
}

#[test]
fn read_mode_selection_is_highlighted() {
    let mut app = App::new(Editor::new("hello world"), "a.md".into());
    app.view = View::Read;
    app.handle(plain('v'));
    app.handle(plain('l'));
    assert!(
        reversed_at(&mut app, 0, 0),
        "read selection should be reversed"
    );
}

#[test]
fn read_mode_yank_copies_the_selected_text() {
    let mut app = App::new(Editor::new("hello world"), "a.md".into());
    app.view = View::Read;
    app.handle(plain('v'));
    for _ in 0..4 {
        app.handle(plain('l'));
    }
    app.handle(plain('y'));
    assert_eq!(app.editor.yanked().as_deref(), Some("hello"));
    assert!(app.read_select.is_none(), "yank ends the selection");
    assert_eq!(app.view, View::Read, "stays in read after copy");
}

#[test]
fn read_mode_capital_y_copies_the_line() {
    let mut app = App::new(Editor::new("hello world"), "a.md".into());
    app.view = View::Read;
    app.handle(plain('Y'));
    assert_eq!(app.editor.yanked().as_deref(), Some("hello world\n"));
}

#[test]
fn read_mode_motions_still_work() {
    let mut app = App::new(Editor::new("# Title\n\nbody"), "a.md".into());
    app.view = View::Read;
    app.handle(plain('j'));
    assert_eq!(app.view, View::Read, "j stays in read mode");
}

#[test]
fn read_mode_editing_keys_still_delegate_to_edit() {
    let mut app = App::new(Editor::new("hello"), "a.md".into());
    app.view = View::Read;
    app.handle(plain('o'));
    assert_eq!(app.view, View::Edit, "o opens a line in edit mode");
}
