mod common;

use v_ide_text::Position;
use v_ide_tui::app::App;

#[test]
fn inactive_shared_windows_render_their_own_horizontal_scroll() {
    let mut app = App::new(common::named_doc("01234567890123456789", "a.md"));
    app.workspace
        .current_mut()
        .editor
        .set_cursor(Position::new(0, 15));
    common::draw(&mut app, 10, 8);
    assert_eq!(app.ui.left, 6);
    app.vsplit(None);
    let frame = common::draw(&mut app, 21, 8);
    let left: String = (0..10)
        .map(|x| frame.grid.cell(x, 0).symbol.as_str())
        .collect();
    assert_eq!(left, "‹78901234›");
    app.focus_next();
    app.workspace
        .current_mut()
        .editor
        .set_cursor(Position::new(0, 0));
    let frame = common::draw(&mut app, 21, 8);
    assert_eq!(frame.grid.cell(12, 0).symbol.as_str(), "7");
}
