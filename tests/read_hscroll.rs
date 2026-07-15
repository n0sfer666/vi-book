use ratatui::backend::TestBackend;
use ratatui::Terminal;

use vi_book::markdown::wrap_mode::WrapMode;
use vi_book::ui::app::App;
use vi_book::ui::view::View;
use vi_book::Editor;

const LINE: &str = "0123456789ABCDEF";
const LEFT: char = '\u{2039}';
const RIGHT: char = '\u{203a}';

fn draw_row0(app: &mut App, w: u16) -> String {
    let backend = TestBackend::new(w, 3);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| vi_book::ui::render::draw(f, app))
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    (0..w).map(|x| buf[(x, 0)].symbol().to_string()).collect()
}

fn read_app() -> App {
    let mut app = App::new(Editor::new(LINE), "a.md".into());
    app.view = View::Read;
    app.wrap = WrapMode::None;
    app
}

#[test]
fn long_line_is_not_wrapped_and_marks_right_overflow() {
    let mut app = read_app();
    let row = draw_row0(&mut app, 10);
    assert_eq!(app.left, 0);
    assert!(row.starts_with("012345678"), "row: {row:?}");
    assert!(row.ends_with(RIGHT), "right overflow marker, row: {row:?}");
}

#[test]
fn cursor_past_edge_scrolls_right_with_both_markers() {
    let mut app = read_app();
    app.read_cursor = (0, 12);
    let row = draw_row0(&mut app, 10);
    assert_eq!(app.left, 3, "left = 12 + 1 - 10");
    assert!(row.starts_with(LEFT), "left marker, row: {row:?}");
    assert!(row.ends_with(RIGHT), "right marker, row: {row:?}");
}

#[test]
fn scrolls_back_to_origin_when_cursor_returns() {
    let mut app = read_app();
    app.read_cursor = (0, 12);
    draw_row0(&mut app, 10);
    assert!(app.left > 0);
    app.read_cursor = (0, 0);
    draw_row0(&mut app, 10);
    assert_eq!(app.left, 0);
}
