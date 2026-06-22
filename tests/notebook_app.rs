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
fn switching_files_with_unsaved_changes_stashes_the_dirty_buffer() {
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
    let stashed_dirty = app
        .buffers
        .iter()
        .find(|p| p.path == "a.md")
        .map(|p| (p.editor.text(), p.saved_text.clone()));
    let a_on_disk = root.join("a.md").exists();
    let _ = fs::remove_dir_all(&root);
    assert_eq!(path, "b.md", "switching is always allowed now");
    assert_eq!(text, "target body", "target buffer is loaded");
    let (a_text, a_saved) = stashed_dirty.expect("dirty buffer is stashed in the pool");
    assert_eq!(a_text, dirty_text, "stashed buffer keeps its edits");
    assert_ne!(a_text, a_saved, "stashed buffer stays dirty");
    assert!(!a_on_disk, "switching never writes to disk");
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
