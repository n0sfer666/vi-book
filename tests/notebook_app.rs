use std::fs;
use std::path::PathBuf;

use vi_book::ui::app::App;
use vi_book::ui::session::Session;
use vi_book::vim::editor::Editor;

fn temp_vault(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("vi-book-app-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

#[test]
fn switching_files_with_unsaved_changes_is_refused() {
    let root = temp_vault("dirty");
    fs::write(root.join("b.md"), "target body").unwrap();
    let (session, _) = Session::open(Some(root.to_str().unwrap())).unwrap();

    let mut app = App::new(Editor::new("hello"), "a.md".into());
    app.session = session;
    app.editor.handle_keys("ix<Esc>");
    let dirty_text = app.editor.text();
    assert_ne!(dirty_text, app.saved_text);

    app.load("b.md");

    let path = app.path.clone();
    let text = app.editor.text();
    let warned = !app.message.is_empty();
    let _ = fs::remove_dir_all(&root);
    assert_eq!(path, "a.md", "must not switch away from a dirty buffer");
    assert_eq!(text, dirty_text, "dirty buffer must be preserved");
    assert!(warned, "must warn about unsaved changes");
}

#[test]
fn switching_files_when_clean_loads_target() {
    let root = temp_vault("clean");
    fs::write(root.join("b.md"), "target body").unwrap();
    let (session, _) = Session::open(Some(root.to_str().unwrap())).unwrap();

    let mut app = App::new(Editor::new("hello"), "a.md".into());
    app.session = session;
    app.load("b.md");

    let text = app.editor.text();
    let path = app.path.clone();
    let _ = fs::remove_dir_all(&root);
    assert_eq!(path, "b.md");
    assert_eq!(text, "target body");
}
