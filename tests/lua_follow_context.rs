mod common;
use std::fs;
use v_ide_editor::document::Document;
use v_ide_tui::{app::App, link_follow::follow_at_cursor};

fn setup(label: &str, command: &str) -> App {
    let root = common::scratch(label);
    fs::write(root.join("a.md"), "see [[target]]").unwrap();
    fs::write(root.join("target.md"), "target").unwrap();
    fs::write(root.join("redirected.md"), "redirected").unwrap();
    let mut app = App::new(Document::new(""));
    common::open_vault(&mut app, &root);
    common::load_lua(
        &mut app,
        label,
        &format!("vibook.on('follow_link',function()vibook.cmd({command:?})end)"),
    );
    app.open_rel("a.md");
    app.workspace.current_mut().editor.set_cursor_index(6);
    app
}

#[test]
fn synchronous_link_navigation_suppresses_the_builtin_fallback() {
    let mut app = setup("follow-inline-edit", "edit redirected.md");
    follow_at_cursor(&mut app);
    assert_eq!(app.workspace.current().name, "redirected.md");
    assert_eq!(app.workspace.current().editor.text(), "redirected");
}

#[test]
fn explicitly_reopening_the_same_buffer_also_handles_the_link() {
    let mut app = setup("follow-inline-same", "edit a.md");
    follow_at_cursor(&mut app);
    assert_eq!(app.workspace.current().name, "a.md");
}
