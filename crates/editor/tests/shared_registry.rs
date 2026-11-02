use v_ide_editor::{document::Document, workspace::Workspace};

fn doc(text: &str, name: &str) -> Document {
    let mut document = Document::new(text);
    document.name = name.into();
    document
}

#[test]
fn visible_and_pooled_references_protect_documents_and_stale_handles_are_rejected() {
    let mut workspace = Workspace::new(doc("first", "a.md"));
    let first = workspace.current_id();
    workspace.split(workspace.current_buffer()).unwrap();
    workspace.open(doc("second", "b.md"));
    let second = workspace.current_buffer();
    assert!(workspace.forget(first).is_none());
    workspace.focus_next();
    assert_eq!(workspace.current().editor.text(), "first");
    let _ = workspace.only();
    assert!(workspace.forget(second.id).is_none());
    let removed = workspace.remove(0).unwrap();
    assert_eq!(removed, second);
    assert!(workspace.doc(second.id).is_none());
    assert!(workspace.split(second).is_none());
    assert!(workspace.put_current(second).is_none());
    assert_eq!(workspace.current_id(), first);
    assert_eq!(workspace.windows().len(), 1);
}

#[test]
fn replacing_an_empty_shared_window_keeps_the_other_reference_alive() {
    let mut workspace = Workspace::new(Document::new(""));
    let original = workspace.current_id();
    workspace.split(workspace.current_buffer()).unwrap();
    workspace.open(doc("second", "b.md"));
    assert!(workspace.doc(original).is_some());
    assert!(workspace.pool().is_empty());
    workspace.focus_next();
    assert_eq!(workspace.current_id(), original);
    workspace.current_mut().editor.handle_keys("idraft<Esc>");
    let _ = workspace.replace(doc("third", "c.md"));
    assert_eq!(workspace.doc(original).unwrap().editor.text(), "draft");
    assert_eq!(workspace.forget(original).unwrap().editor.text(), "draft");
}

#[test]
fn adding_hidden_buffers_preserves_insertion_order_until_they_are_visited() {
    let mut workspace = Workspace::new(doc("current", "current.md"));
    let first = workspace.add(doc("first", "a.md"));
    let second = workspace.add(doc("second", "b.md"));
    let ids: Vec<_> = workspace.pool().iter().map(|buffer| buffer.id).collect();
    assert_eq!(ids, vec![first, second]);
}
