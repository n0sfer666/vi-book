mod common;

use std::fs;
use std::path::{Path, PathBuf};

use v_ide_editor::path::rel_path;

use v_ide_tui::app::App;
use v_ide_tui::link_follow::follow_at_cursor;

fn temp_vault(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("vi-book-links-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

fn app_for(root: &std::path::Path) -> App {
    let mut app = App::new(common::named_doc("", ""));
    common::open_vault(&mut app, root);
    app
}

#[test]
fn following_a_code_link_requests_the_external_editor() {
    let root = temp_vault("code");
    fs::write(root.join("script.rs"), "fn main() {}\n").unwrap();
    fs::write(root.join("a.md"), "see [[script.rs]]\n").unwrap();
    let mut app = app_for(&root);
    app.open_rel("a.md");
    app.workspace.current_mut().editor.set_cursor_index(6);
    follow_at_cursor(&mut app);
    let request = app.external_request.clone();
    let _ = fs::remove_dir_all(&root);
    assert_eq!(request, Some(root.join("script.rs")));
}

#[test]
fn following_a_markdown_link_opens_in_vi_book() {
    let root = temp_vault("md");
    fs::write(root.join("target.md"), "# Target\n").unwrap();
    fs::write(root.join("a.md"), "see [[target.md]]\n").unwrap();
    let mut app = app_for(&root);
    app.open_rel("a.md");
    app.workspace.current_mut().editor.set_cursor_index(6);
    follow_at_cursor(&mut app);
    let path = app.workspace.current().name.clone();
    let external = app.external_request.clone();
    let _ = fs::remove_dir_all(&root);
    assert_eq!(path, "target.md");
    assert!(external.is_none());
}

#[test]
fn following_a_markdown_link_to_a_note_opens_in_vi_book() {
    let root = temp_vault("mdlink");
    fs::write(root.join("target.md"), "# Target\n").unwrap();
    fs::write(root.join("a.md"), "see [Target](target.md)\n").unwrap();
    let mut app = app_for(&root);
    app.open_rel("a.md");
    app.workspace.current_mut().editor.set_cursor_index(6);
    follow_at_cursor(&mut app);
    let path = app.workspace.current().name.clone();
    let external = app.external_request.clone();
    let _ = fs::remove_dir_all(&root);
    assert_eq!(path, "target.md");
    assert!(external.is_none());
}

#[test]
fn following_a_markdown_link_to_code_requests_the_external_editor() {
    let root = temp_vault("mdcode");
    fs::write(root.join("script.rs"), "fn main() {}\n").unwrap();
    fs::write(root.join("a.md"), "see [s](script.rs)\n").unwrap();
    let mut app = app_for(&root);
    app.open_rel("a.md");
    app.workspace.current_mut().editor.set_cursor_index(5);
    follow_at_cursor(&mut app);
    let request = app.external_request.clone();
    let _ = fs::remove_dir_all(&root);
    assert_eq!(request, Some(root.join("script.rs")));
}

#[test]
fn following_a_markdown_link_to_a_url_requests_the_browser() {
    let root = temp_vault("mdurl");
    fs::write(root.join("a.md"), "see [site](https://example.com)\n").unwrap();
    let mut app = app_for(&root);
    app.open_rel("a.md");
    app.workspace.current_mut().editor.set_cursor_index(6);
    follow_at_cursor(&mut app);
    let url = app.url_request.clone();
    let path = app.workspace.current().name.clone();
    let _ = fs::remove_dir_all(&root);
    assert_eq!(url, Some("https://example.com".to_string()));
    assert_eq!(path, "a.md");
}

#[test]
fn following_a_bare_url_requests_the_browser() {
    let root = temp_vault("bareurl");
    fs::write(root.join("a.md"), "visit https://example.com today\n").unwrap();
    let mut app = app_for(&root);
    app.open_rel("a.md");
    app.workspace.current_mut().editor.set_cursor_index(8);
    follow_at_cursor(&mut app);
    let url = app.url_request.clone();
    let _ = fs::remove_dir_all(&root);
    assert_eq!(url, Some("https://example.com".to_string()));
}

#[test]
fn following_a_backtick_filename_opens_when_it_resolves() {
    let root = temp_vault("tick");
    fs::write(root.join("target.md"), "# Target\n").unwrap();
    fs::write(root.join("a.md"), "see `target.md` here\n").unwrap();
    let mut app = app_for(&root);
    app.open_rel("a.md");
    app.workspace.current_mut().editor.set_cursor_index(7);
    follow_at_cursor(&mut app);
    let path = app.workspace.current().name.clone();
    let _ = fs::remove_dir_all(&root);
    assert_eq!(path, "target.md");
}

#[test]
fn following_a_backtick_non_file_does_nothing() {
    let root = temp_vault("nofile");
    fs::write(root.join("a.md"), "run `cargo build` now\n").unwrap();
    let mut app = app_for(&root);
    app.open_rel("a.md");
    app.workspace.current_mut().editor.set_cursor_index(7);
    follow_at_cursor(&mut app);
    let path = app.workspace.current().name.clone();
    let external = app.external_request.clone();
    let url = app.url_request.clone();
    let _ = fs::remove_dir_all(&root);
    assert_eq!(path, "a.md");
    assert!(external.is_none());
    assert!(url.is_none());
}

#[test]
fn lua_follow_error_falls_back_to_core_follow() {
    let root = temp_vault("luaerr");
    fs::write(root.join("target.md"), "# Target\n").unwrap();
    fs::write(root.join("a.md"), "see [[target]]\n").unwrap();
    let mut app = app_for(&root);
    common::load_lua(
        &mut app,
        "links-luaerr",
        "vibook.on('follow_link', function() error('boom') end)",
    );
    app.open_rel("a.md");
    app.workspace.current_mut().editor.set_cursor_index(6);
    follow_at_cursor(&mut app);
    let path = app.workspace.current().name.clone();
    let message = app.ui.message.clone();
    let _ = fs::remove_dir_all(&root);
    assert_eq!(path, "target.md");
    assert!(message.starts_with("plugin error:"), "{message}");
}

#[test]
fn following_an_absolute_note_outside_vault_keeps_its_path() {
    let root = temp_vault("abs");
    let outside = temp_vault("abs-outside");
    let target = outside.join("x.md");
    fs::write(&target, "# X\n").unwrap();
    fs::write(root.join("a.md"), format!("see [[{}]]\n", target.display())).unwrap();
    let mut app = app_for(&root);
    app.open_rel("a.md");
    app.workspace.current_mut().editor.set_cursor_index(6);
    follow_at_cursor(&mut app);
    let path = app.workspace.current().name.clone();
    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&outside);
    assert_eq!(path, target.to_string_lossy());
}

#[test]
fn rel_path_outside_root_is_returned_as_is() {
    assert_eq!(
        rel_path(Path::new("/vault"), Path::new("/abs/x.md")),
        "/abs/x.md"
    );
    assert_eq!(
        rel_path(Path::new("/vault"), Path::new("/vault/d/x.md")),
        "d/x.md"
    );
}
