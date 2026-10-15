mod common;

use v_ide_view::Rect;

use v_ide_tui::app::App;
use v_ide_tui::layout::{column_width, text_column};
use v_ide_tui::view::View;

#[test]
fn wide_terminal_uses_full_width() {
    let area = Rect {
        x: 0,
        y: 0,
        w: 100,
        h: 24,
    };
    assert_eq!(column_width(area), 100);
    let col = text_column(area);
    assert_eq!(col.w, 100);
    assert_eq!(col.x, 0);
}

#[test]
fn narrow_terminal_uses_full_width_without_offset() {
    let area = Rect {
        x: 0,
        y: 0,
        w: 40,
        h: 24,
    };
    let col = text_column(area);
    assert_eq!(col.x, 0);
    assert_eq!(col.w, 40);
}

#[test]
fn column_respects_area_origin_and_height() {
    let area = Rect {
        x: 10,
        y: 2,
        w: 100,
        h: 20,
    };
    let col = text_column(area);
    assert_eq!(col.x, 10);
    assert_eq!(col.y, 2);
    assert_eq!(col.w, 100);
    assert_eq!(col.h, 20);
}

#[test]
fn zero_width_clamps_to_one() {
    let area = Rect {
        x: 0,
        y: 0,
        w: 0,
        h: 24,
    };
    assert_eq!(column_width(area), 1);
}

#[test]
fn edit_content_starts_at_origin_not_centered() {
    let mut app = App::new(common::named_doc("hello", "a.md"));
    app.ui.read.view = View::Edit;
    let row = common::draw(&mut app, 100, 24).grid.row_text(0);
    assert!(
        row.starts_with("hello"),
        "edit content should start at column 0, got: {row:?}"
    );
}
