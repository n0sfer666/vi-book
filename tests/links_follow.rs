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
