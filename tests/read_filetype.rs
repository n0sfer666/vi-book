mod common;

use std::fs;

use v_ide_markdown::wrap_mode::WrapMode;
use v_ide_tui::actions;
use v_ide_tui::app::App;
use v_ide_tui::view::View;

fn vault_app(label: &str) -> App {
    let root = common::temp_root(label);
    fs::write(root.join("a.md"), "# A").unwrap();
    fs::write(root.join("main.rs"), "fn main() {}").unwrap();
    let mut app = App::new(common::named_doc("", ""));
    common::open_vault(&mut app, &root);
    app
}

#[test]
fn markdown_opens_in_read_view_and_activates_module() {
    let mut app = vault_app("ft-md");
    app.open_rel("a.md");
    assert_eq!(app.ui.read.view, View::Read);
    assert_eq!(common::wrap(&app), Some(WrapMode::All));
}

#[test]
fn other_filetypes_open_in_edit_view() {
    let mut app = vault_app("ft-rs");
    app.open_rel("main.rs");
    assert_eq!(app.ui.read.view, View::Edit);
    assert_eq!(common::wrap(&app), None);
}

#[test]
fn toggle_view_outside_markdown_keeps_edit_with_message() {
    let mut app = vault_app("ft-toggle-rs");
    app.open_rel("main.rs");
    actions::toggle_view(&mut app);
    assert_eq!(app.ui.read.view, View::Edit);
    assert_eq!(app.ui.message, "read view: markdown only");
}

#[test]
fn toggle_view_in_markdown_switches_both_ways() {
    let mut app = vault_app("ft-toggle-md");
    app.open_rel("a.md");
    actions::toggle_view(&mut app);
    assert_eq!(app.ui.read.view, View::Edit);
    actions::toggle_view(&mut app);
    assert_eq!(app.ui.read.view, View::Read);
}

#[test]
fn vsplit_of_other_filetype_opens_in_edit_view() {
    let mut app = vault_app("ft-vsplit");
    app.open_rel("a.md");
    app.vsplit(Some("main.rs"));
    assert_eq!(app.ui.read.view, View::Edit);
}
