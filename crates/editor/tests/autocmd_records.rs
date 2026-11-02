#[path = "autocmd/common.rs"]
mod common;

use common::{add, user};
use v_ide_editor::autocmd::{Attributes, Filter, GroupFilter, Registry, Selector};
use v_ide_editor::{
    document::Document,
    workspace::{DocId, Workspace},
};

fn doc(index: usize) -> DocId {
    let mut workspace = Workspace::new(Document::new("buffer"));
    let mut id = workspace.current_id();
    for _ in 0..index {
        id = workspace.add(Document::new("buffer"));
    }
    id
}

#[test]
fn one_api_definition_expands_duplicates_and_once_consumes_each_record_separately() {
    let mut registry = Registry::new();
    let mut definition = registry
        .define(
            Attributes {
                once: true,
                ..Attributes::default()
            },
            "call",
        )
        .unwrap();
    let id = definition.id().unwrap();
    for _ in 0..4 {
        definition.push(user(), Selector::File("*".into()));
    }
    drop(definition);
    let rows = registry.entries(&Filter::default());
    assert_eq!(rows.len(), 4);
    assert!(rows.iter().all(|row| row.id == Some(id)));
    let keys = registry.candidates(user(), doc(0), GroupFilter::All);
    let trace: Vec<_> = keys
        .into_iter()
        .filter_map(|key| registry.begin(key))
        .map(|row| row.action)
        .collect();
    assert_eq!(trace, ["call", "call", "call", "call"]);
    assert!(registry.entries(&Filter::default()).is_empty());
}

#[test]
fn deleting_one_callback_record_preserves_other_patterns_with_the_same_public_id() {
    let mut registry = Registry::new();
    let mut definition = registry.define(Attributes::default(), "call").unwrap();
    let id = definition.id().unwrap();
    definition.push(user(), Selector::File("one".into()));
    definition.push(user(), Selector::File("two".into()));
    drop(definition);
    let first = registry.candidates(user(), doc(0), GroupFilter::All)[0];
    assert!(registry.begin(first).is_some());
    assert!(registry.remove_record(first));
    let rows = registry.entries(&Filter::default());
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, Some(id));
    assert_eq!(rows[0].selector, Selector::File("two".into()));
    registry.delete(id).unwrap();
    assert!(registry.entries(&Filter::default()).is_empty());
}

#[test]
fn deletion_invalidates_pending_handles_and_buffer_cleanup_keeps_filename_rules() {
    let mut registry = Registry::new();
    let first = add(
        &mut registry,
        user(),
        Selector::Buffer(doc(1)),
        Attributes::default(),
        "local",
    );
    add(
        &mut registry,
        user(),
        Selector::File("*".into()),
        Attributes::default(),
        "global",
    );
    let pending = registry.candidates(user(), doc(1), GroupFilter::All);
    assert_eq!(pending.len(), 2);
    assert_eq!(
        registry.candidates(user(), doc(2), GroupFilter::All).len(),
        1
    );
    registry.delete(first).unwrap();
    assert!(registry.begin(pending[0]).is_none());
    assert_eq!(registry.begin(pending[1]).unwrap().action, "global");
    add(
        &mut registry,
        user(),
        Selector::Buffer(doc(1)),
        Attributes::default(),
        "again",
    );
    registry.forget_buffer(doc(1));
    assert_eq!(registry.entries(&Filter::default()).len(), 1);
}
