mod common;

use v_ide_text::Position;
use v_ide_tui::view::View;
use v_ide_tui::{actions, app::App, command_line};

#[test]
fn configuration_defaults_show_numbers_in_edit_and_hide_them_in_read() {
    let mut app = App::new(common::named_doc("Hello", "a.md"));
    common::load_lua(&mut app, "number-defaults", "");
    app.ui.read.view = View::Edit;
    assert!(
        common::draw(&mut app, 24, 8)
            .grid
            .row_text(0)
            .starts_with("  1 Hello")
    );
    actions::toggle_view(&mut app);
    let frame = common::draw(&mut app, 24, 8);
    assert!(frame.grid.row_text(0).starts_with("Hello"));
    assert_eq!(frame.cursor, Some((0, 0)));
}

#[test]
fn read_numbers_are_independent_of_edit_number_and_apply_to_inactive_splits() {
    let mut app = App::new(common::named_doc("Hello", "a.md"));
    common::load_lua(
        &mut app,
        "number-independent",
        "vibook.opt.number = false; vibook.opt.read_number = true",
    );
    app.ui.read.view = View::Edit;
    assert!(
        common::draw(&mut app, 24, 8)
            .grid
            .row_text(0)
            .starts_with("Hello")
    );
    actions::toggle_view(&mut app);
    app.vsplit(None);
    let frame = common::draw(&mut app, 49, 8);
    assert!(frame.grid.row_text(0).starts_with("  1 Hello"));
    assert_eq!(frame.grid.cell(27, 0).symbol.as_str(), "1");
    assert_eq!(frame.cursor, Some((29, 0)));
    app.focus_next();
    assert_eq!(common::draw(&mut app, 49, 8).cursor, Some((4, 0)));
}

#[test]
fn read_numbers_use_wrapped_display_rows_and_motion_keeps_the_source_mapping() {
    let mut app = App::new(common::named_doc("one two three four", "a.md"));
    common::load_lua(
        &mut app,
        "number-wrap",
        "vibook.opt.read_number = true; vibook.opt.wrap = 'text'",
    );
    command_line::run(&mut app, "wrap text");
    actions::toggle_view(&mut app);
    let frame = common::draw(&mut app, 14, 6);
    assert!(frame.grid.row_text(0).starts_with("  1 one two"));
    assert!(frame.grid.row_text(1).starts_with("  2 three four"));
    common::feed::press(&mut app, "Gi");
    assert_eq!(app.ui.read.view, View::Edit);
    assert_eq!(app.workspace.current().editor.cursor(), Position::new(0, 8));
}

#[test]
fn config_reload_preserves_runtime_edit_choice_unless_explicitly_reassigned() {
    let mut app = App::new(common::named_doc("one", "a.md"));
    common::load_lua(&mut app, "number-first-load", "");
    command_line::run(&mut app, "set nonu");
    common::load_lua(&mut app, "number-reload", "vibook.opt.read_number = true");
    command_line::run(&mut app, "set nu?");
    assert_eq!(app.ui.message, "nonumber");
}

#[test]
fn inactive_numbered_read_pane_keeps_its_horizontal_viewport_and_markers() {
    let mut app = App::new(common::named_doc("abcdefghijklmnopqrstuvwxyz", "a.md"));
    common::load_lua(
        &mut app,
        "number-read-hscroll",
        "vibook.opt.read_number = true; vibook.opt.wrap = 'none'",
    );
    actions::toggle_view(&mut app);
    common::feed::press(&mut app, "$");
    app.vsplit(None);
    common::draw(&mut app, 41, 6);
    app.focus_next();
    let frame = common::draw(&mut app, 41, 6);
    assert_eq!(frame.grid.cell(4, 0).symbol.as_str(), "‹");
    assert_eq!(frame.grid.cell(25, 0).symbol.as_str(), "‹");
    assert_eq!(frame.grid.cell(26, 0).symbol.as_str(), "l");
}
