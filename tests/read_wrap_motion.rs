mod common;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use v_ide_tui::app::App;
use v_ide_tui::read_view::handle_key;
use v_ide_tui::view::View;

const WIDTH: usize = 7;

fn read_app(text: &str, cursor: (usize, usize)) -> App {
    let mut app = App::new(common::named_doc(text, "a.md"));
    app.ui.read.view = View::Read;
    app.ui.read.read_cursor = cursor;
    app
}

fn press(app: &mut App, keys: &str) -> (usize, usize) {
    for c in keys.chars() {
        handle_key(
            app,
            KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE),
            WIDTH,
        );
    }
    app.ui.read.read_cursor
}

#[test]
fn dollar_goes_to_end_of_logical_line_across_wrap() {
    let mut app = read_app("aaa bbb ccc ddd\n\nzzz", (0, 0));
    assert_eq!(press(&mut app, "$"), (1, 6));
}

#[test]
fn zero_goes_to_start_of_logical_line_across_wrap() {
    let mut app = read_app("aaa bbb ccc ddd\n\nzzz", (1, 4));
    assert_eq!(press(&mut app, "0"), (0, 0));
}

#[test]
fn g_dollar_goes_to_end_of_visual_line() {
    let mut app = read_app("aaa bbb ccc ddd\n\nzzz", (0, 0));
    assert_eq!(press(&mut app, "g$"), (0, 6));
}

#[test]
fn g_zero_goes_to_start_of_visual_line() {
    let mut app = read_app("aaa bbb ccc ddd\n\nzzz", (1, 4));
    assert_eq!(press(&mut app, "g0"), (1, 0));
}

#[test]
fn dollar_stops_at_soft_line_break() {
    let mut app = read_app("aaa\nbbb", (0, 0));
    assert_eq!(press(&mut app, "$"), (0, 2));
}
