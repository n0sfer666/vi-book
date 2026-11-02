mod common;

use v_ide_text::Position;
use v_ide_tui::view::View;
use v_ide_tui::{app::App, command_line};

#[test]
fn number_gutter_moves_text_and_cursor_and_leaves_eof_rows_unnumbered() {
    let mut app = App::new(common::named_doc("alpha\nbeta", "a.txt"));
    app.ui.read.view = View::Edit;
    command_line::run(&mut app, "setlocal nu");
    let frame = common::draw(&mut app, 20, 8);
    assert!(frame.grid.row_text(0).starts_with("  1 alpha"));
    assert!(frame.grid.row_text(1).starts_with("  2 beta"));
    assert!(frame.grid.row_text(2).trim().is_empty());
    assert_eq!(frame.cursor, Some((4, 0)));
    command_line::run(&mut app, "setlocal nonu");
    let frame = common::draw(&mut app, 20, 8);
    assert!(frame.grid.row_text(0).starts_with("alpha"));
    assert_eq!(frame.cursor, Some((0, 0)));
}

#[test]
fn inactive_split_renders_its_own_number_setting() {
    let mut app = App::new(common::named_doc("alpha\nbeta", "a.txt"));
    app.ui.read.view = View::Edit;
    command_line::run(&mut app, "setlocal nu");
    app.vsplit(None);
    command_line::run(&mut app, "setlocal nonu");
    let frame = common::draw(&mut app, 41, 8);
    assert!(frame.grid.row_text(0).starts_with("  1 alpha"));
    assert_eq!(frame.grid.cell(21, 0).symbol.as_str(), "a");
    assert_eq!(frame.cursor, Some((21, 0)));
    app.focus_next();
    let frame = common::draw(&mut app, 41, 8);
    assert_eq!(frame.cursor, Some((4, 0)));
    assert_eq!(frame.grid.cell(21, 0).symbol.as_str(), "a");
}

#[test]
fn number_gutter_participates_in_horizontal_scroll_and_vertical_line_labels() {
    let mut app = App::new(common::named_doc("abcdefghijklmnop\nx\ny\nz", "a.txt"));
    app.ui.read.view = View::Edit;
    command_line::run(&mut app, "setlocal nu");
    app.workspace
        .current_mut()
        .editor
        .set_cursor(Position::new(0, 12));
    let frame = common::draw(&mut app, 10, 4);
    assert!(frame.grid.row_text(0).starts_with("  1 "));
    assert_eq!(app.ui.left, 7);
    assert_eq!(frame.cursor, Some((9, 0)));
    app.workspace
        .current_mut()
        .editor
        .set_cursor(Position::new(3, 0));
    let frame = common::draw(&mut app, 10, 4);
    assert_eq!(app.ui.read.top, 2);
    assert!(frame.grid.row_text(0).starts_with("  3 y"));
    assert!(frame.grid.row_text(1).starts_with("  4 z"));
}

#[test]
fn number_width_grows_with_buffer_length_and_tiny_windows_still_draw() {
    let source = vec!["x"; 1000].join("\n");
    let mut app = App::new(common::named_doc(&source, "a.txt"));
    app.ui.read.view = View::Edit;
    command_line::run(&mut app, "setlocal nu");
    let frame = common::draw(&mut app, 15, 5);
    assert!(frame.grid.row_text(0).starts_with("   1 x"));
    assert_eq!(frame.cursor, Some((5, 0)));
    for width in 0..=5 {
        let frame = common::draw(&mut app, width, 5);
        if width > 0 {
            assert!(frame.cursor.is_some_and(|(x, _)| x < width));
        }
    }
}
