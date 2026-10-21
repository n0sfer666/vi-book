mod common;

use std::fs;
use std::path::PathBuf;

use v_ide_editor::document::Document;
use v_ide_notebook::daily::daily_path;
use v_ide_notebook::date::Date;
use v_ide_tui::app::App;
use v_ide_tui::command_line;
use v_ide_vault::module::VaultModule;

fn temp_vault(name: &str) -> PathBuf {
    common::scratch(&format!("app-{name}"))
}

#[test]
fn switching_files_with_unsaved_changes_stashes_the_dirty_buffer() {
    let root = temp_vault("dirty");
    fs::write(root.join("b.md"), "target body").unwrap();

    let mut app = App::new(common::named_doc("hello", "a.md"));
    common::open_vault(&mut app, &root);
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

    let mut app = App::new(common::named_doc("hello", "a.md"));
    common::open_vault(&mut app, &root);
    app.load("b.md");

    let text = app.workspace.current().editor.text();
    let path = app.workspace.current().name.clone();
    let _ = fs::remove_dir_all(&root);
    assert_eq!(path, "b.md");
    assert_eq!(text, "target body");
}

#[test]
fn today_command_opens_daily_and_vault_indexes_it() {
    let root = temp_vault("today");
    let mut app = App::new(Document::new(""));
    common::open_vault(&mut app, &root);
    let before = Date::today();
    command_line::run(&mut app, "today");
    let after = Date::today();
    let rel = app.workspace.current().name.clone();
    assert!(
        [before, after].iter().any(|date| daily_path(*date) == rel),
        "opened {rel}"
    );
    let items = app
        .modules
        .get::<VaultModule>()
        .and_then(|vault| vault.items().ok())
        .unwrap_or_default();
    assert_eq!(items, vec![rel]);
    let _ = fs::remove_dir_all(&root);
}
