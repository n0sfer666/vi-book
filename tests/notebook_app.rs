mod common;

use std::fs;
use std::path::PathBuf;

use v_ide_tui::app::App;
use v_ide_tui::session::Session;

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

    let mut app = App::new(common::named_doc("hello", "a.md"));
    app.session = session;
    app.workspace.current_mut().editor.handle_keys("ix<Esc>");
    let dirty_text = app.workspace.current().editor.text();
    assert!(app.workspace.current().is_dirty());

    app.load("b.md");

    let path = app.workspace.current().name.clone();
    let text = app.workspace.current().editor.text();
    let stashed_dirty = app
        .workspace
        .pool()
        .iter()
        .find(|b| b.doc.name == "a.md")
        .map(|b| (b.doc.editor.text(), b.doc.is_dirty()));
    let a_on_disk = root.join("a.md").exists();
    let _ = fs::remove_dir_all(&root);
    assert_eq!(path, "b.md", "switching is always allowed now");
    assert_eq!(text, "target body", "target buffer is loaded");
    let (a_text, a_dirty) = stashed_dirty.expect("dirty buffer is stashed in the pool");
    assert_eq!(a_text, dirty_text, "stashed buffer keeps its edits");
    assert!(a_dirty, "stashed buffer stays dirty");
    assert!(!a_on_disk, "switching never writes to disk");
}

#[test]
fn switching_files_when_clean_loads_target() {
    let root = temp_vault("clean");
    fs::write(root.join("b.md"), "target body").unwrap();
    let (session, _) = Session::open(Some(root.to_str().unwrap())).unwrap();

    let mut app = App::new(common::named_doc("hello", "a.md"));
    app.session = session;
    app.load("b.md");

    let text = app.workspace.current().editor.text();
    let path = app.workspace.current().name.clone();
    let _ = fs::remove_dir_all(&root);
    assert_eq!(path, "b.md");
    assert_eq!(text, "target body");
}
