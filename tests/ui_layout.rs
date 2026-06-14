use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;

use vi_book::Editor;
use vi_book::ui::app::App;
use vi_book::ui::layout::{column_width, text_column};
use vi_book::ui::view::View;

#[test]
fn wide_terminal_centers_column() {
    let area = Rect::new(0, 0, 100, 24);
    assert_eq!(column_width(area), 90);
    let col = text_column(area);
    assert_eq!(col.width, 90);
    assert_eq!(col.x, 5);
}

#[test]
fn narrow_terminal_uses_full_width_without_offset() {
    let area = Rect::new(0, 0, 40, 24);
    let col = text_column(area);
    assert_eq!(col.x, 0);
    assert_eq!(col.width, 40);
}

#[test]
fn column_respects_area_origin_and_height() {
    let area = Rect::new(10, 2, 100, 20);
    let col = text_column(area);
    assert_eq!(col.x, 15);
    assert_eq!(col.y, 2);
    assert_eq!(col.height, 20);
}

#[test]
fn edit_content_is_centered_like_read() {
    let mut app = App::new(Editor::new("hello"), "a.md".into());
    app.view = View::Edit;
    let backend = TestBackend::new(100, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| vi_book::ui::render::draw(frame, &mut app))
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let row: String = (0..100).map(|x| buf[(x, 0)].symbol().to_string()).collect();
    assert!(
        row.starts_with("     hello"),
        "edit content should be centered at column 5, got: {row:?}"
    );
}
