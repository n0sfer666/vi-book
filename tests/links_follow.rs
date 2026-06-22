use std::fs;
use std::path::PathBuf;

use vi_book::Editor;
use vi_book::lua::defaults::LINKS;
use vi_book::lua::host::PluginHost;
use vi_book::ui::app::App;
use vi_book::ui::link_follow::follow_at_cursor;
use vi_book::ui::session::Session;

fn temp_vault(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("vi-book-links-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

fn app_for(root: &std::path::Path) -> App {
    let (session, _) = Session::open(Some(root.to_str().unwrap())).unwrap();
    let mut app = App::new(Editor::new(""), String::new());
    app.session = session;
    let host = PluginHost::new().unwrap();
    host.load(LINKS).unwrap();
    host.set_vault(app.session.items(), app.session.index.tags());
    app.host = Some(host);
    app
}

#[test]
fn following_a_code_link_requests_the_external_editor() {
    let root = temp_vault("code");
    fs::write(root.join("script.rs"), "fn main() {}\n").unwrap();
    fs::write(root.join("a.md"), "see [[script.rs]]\n").unwrap();
    let mut app = app_for(&root);
    app.open_rel("a.md");
    app.editor.set_cursor_index(6);
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
    app.editor.set_cursor_index(6);
    follow_at_cursor(&mut app);
    let path = app.path.clone();
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
    app.editor.set_cursor_index(6);
    follow_at_cursor(&mut app);
    let path = app.path.clone();
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
    app.editor.set_cursor_index(5);
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
    app.editor.set_cursor_index(6);
    follow_at_cursor(&mut app);
    let url = app.url_request.clone();
    let path = app.path.clone();
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
    app.editor.set_cursor_index(8);
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
    app.editor.set_cursor_index(7);
    follow_at_cursor(&mut app);
    let path = app.path.clone();
    let _ = fs::remove_dir_all(&root);
    assert_eq!(path, "target.md");
}

#[test]
fn following_a_backtick_non_file_does_nothing() {
    let root = temp_vault("nofile");
    fs::write(root.join("a.md"), "run `cargo build` now\n").unwrap();
    let mut app = app_for(&root);
    app.open_rel("a.md");
    app.editor.set_cursor_index(7);
    follow_at_cursor(&mut app);
    let path = app.path.clone();
    let external = app.external_request.clone();
    let url = app.url_request.clone();
    let _ = fs::remove_dir_all(&root);
    assert_eq!(path, "a.md");
    assert!(external.is_none());
    assert!(url.is_none());
}
