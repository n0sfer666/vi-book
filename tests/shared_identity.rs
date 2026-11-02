mod common;

use std::fs;
use v_ide_editor::document::Document;
use v_ide_tui::app::App;

fn app(label: &str) -> (App, std::path::PathBuf) {
    let root = common::scratch(label);
    fs::write(root.join("a.txt"), "alpha").unwrap();
    let mut doc = Document::open(root.join("a.txt")).unwrap();
    doc.name = "a.txt".into();
    let mut app = App::new(doc);
    app.workspace.set_cwd(root.clone());
    (app, root)
}

fn shared_target(app: &mut App, target: &str) {
    let id = app.workspace.current_id();
    app.workspace.current_mut().editor.handle_keys("Ax<Esc>");
    app.vsplit(Some(target));
    assert_eq!(app.workspace.current_id(), id);
    assert_eq!(app.workspace.current().editor.text(), "alphax");
    assert_eq!(app.workspace.documents().count(), 1);
}

#[test]
fn parent_directory_alias_reuses_one_resident_file() {
    let (mut app, root) = app("shared-parent-alias");
    fs::create_dir(root.join("nested")).unwrap();
    shared_target(&mut app, "nested/../a.txt");
}

#[cfg(unix)]
#[test]
fn symbolic_link_alias_reuses_one_resident_file() {
    let (mut app, root) = app("shared-symlink");
    std::os::unix::fs::symlink("a.txt", root.join("link.txt")).unwrap();
    shared_target(&mut app, "link.txt");
}

#[test]
#[cfg(unix)]
fn hard_link_alias_reuses_one_resident_file() {
    let (mut app, root) = app("shared-hardlink");
    fs::hard_link(root.join("a.txt"), root.join("link.txt")).unwrap();
    shared_target(&mut app, "link.txt");
}

#[test]
fn a_real_file_is_distinct_from_a_same_named_bundled_document() {
    let root = common::scratch("shared-bundled-identity");
    fs::write(root.join("a.txt"), "alpha").unwrap();
    let mut app = App::new(common::named_doc("bundled", "a.txt"));
    app.workspace.set_cwd(root);
    let id = app.workspace.current_id();
    app.open_rel("a.txt");
    assert_ne!(app.workspace.current_id(), id);
    assert_eq!(app.workspace.current().editor.text(), "alpha");
}

#[test]
fn relative_workspace_roots_resolve_file_identity_once() {
    let root = common::scratch("shared-relative-root");
    fs::write(root.join("a.txt"), "alpha").unwrap();
    fs::create_dir(root.join("nested")).unwrap();
    let base = std::env::current_dir().unwrap();
    let relative = root.strip_prefix(base).unwrap();
    let mut doc = Document::open(relative.join("a.txt")).unwrap();
    doc.name = "a.txt".into();
    let mut app = App::new(doc);
    app.workspace.set_cwd(relative.to_path_buf());
    shared_target(&mut app, "nested/../a.txt");
}

#[test]
fn deleted_resident_files_are_still_reused_through_parent_aliases() {
    let (mut app, root) = app("shared-deleted-alias");
    fs::create_dir(root.join("nested")).unwrap();
    fs::write(root.join("b.txt"), "beta").unwrap();
    let id = app.workspace.current_id();
    app.workspace.current_mut().editor.handle_keys("Ax<Esc>");
    app.open_rel("b.txt");
    fs::remove_file(root.join("a.txt")).unwrap();
    app.vsplit(Some("nested/../a.txt"));
    assert_eq!(app.workspace.current_id(), id);
    assert_eq!(app.workspace.current().editor.text(), "alphax");
}
