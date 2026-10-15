mod common;

use v_ide_tui::app::App;
use v_ide_tui::view::View;

const LINE: &str = "0123456789ABCDEF";
const LEFT: char = '\u{2039}';
const RIGHT: char = '\u{203a}';

fn draw_row0(app: &mut App, w: u16) -> String {
    common::draw(app, w, 3).grid.row_text(0)
}

fn edit_app() -> App {
    let mut app = App::new(common::named_doc(LINE, "a.md"));
    app.ui.read.view = View::Edit;
    app
}

#[test]
fn long_line_marks_right_overflow_in_edit() {
    let mut app = edit_app();
    let row = draw_row0(&mut app, 10);
    assert_eq!(app.ui.left, 0);
    assert!(row.starts_with("012345678"), "row: {row:?}");
    assert!(row.ends_with(RIGHT), "right marker, row: {row:?}");
}

#[test]
fn cursor_past_edge_scrolls_edit_with_both_markers() {
    let mut app = edit_app();
    app.workspace.current_mut().editor.handle_keys("12l");
    let row = draw_row0(&mut app, 10);
    assert!(app.ui.left > 0, "left = {}", app.ui.left);
    assert!(row.starts_with(LEFT), "left marker, row: {row:?}");
    assert!(row.ends_with(RIGHT), "right marker, row: {row:?}");
}
