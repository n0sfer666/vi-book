mod common;

use std::fs;
use v_ide_editor::document::Document;
use v_ide_tui::app::App;

#[test]
fn closing_a_shared_current_buffer_does_not_delete_an_unrelated_dirty_buffer() {
    let root = common::scratch("shared-close-identity");
    for (name, text) in [("a.txt", "alpha"), ("b.txt", "beta"), ("c.txt", "gamma")] {
        fs::write(root.join(name), text).unwrap();
    }
    let mut doc = Document::open(root.join("a.txt")).unwrap();
    doc.name = "a.txt".into();
    let mut app = App::new(doc);
    app.workspace.set_cwd(root);
    app.vsplit(None);
    app.open_rel("c.txt");
    let dirty = app.workspace.current_id();
    app.workspace.current_mut().editor.handle_keys("ix<Esc>");
    app.open_rel("b.txt");
    app.open_rel("a.txt");
    app.close_buffer(0);
    assert_eq!(app.workspace.current().name, "b.txt");
    assert_eq!(app.workspace.doc(dirty).unwrap().editor.text(), "xgamma");
    assert!(app.workspace.doc(dirty).unwrap().is_dirty());
    assert!(app.workspace.pool().iter().any(|buffer| buffer.id == dirty));
    app.focus_next();
    assert_eq!(app.workspace.current().editor.text(), "alpha");
}
