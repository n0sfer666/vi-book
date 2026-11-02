#[path = "autocmd/bus_common.rs"]
mod common;

use common::{add, event, user};
use std::{cell::RefCell, rc::Rc};
use v_ide_editor::{
    autocmd::{
        Action, Attributes, Bus, Filter, GroupFilter, Invocation, Names, Occurrence, Selector,
        Source,
    },
    document::Document,
    execution::Context,
    workspace::Workspace,
};

fn trace_action(trace: &Rc<RefCell<Vec<String>>>, label: &'static str) -> Action {
    let trace = trace.clone();
    Action::callback(move |_: &Context<'_>, _: &Invocation<'_>| {
        trace.borrow_mut().push(label.to_owned());
        Ok(false)
    })
}

#[test]
fn dispatch_matches_compiled_patterns_group_filters_and_optional_buffers() {
    let mut workspace = Workspace::new(Document::new(""));
    let buffer = workspace.current_id();
    let cx = Context::new(&mut workspace);
    let bus = Bus::new(false);
    let trace = Rc::new(RefCell::new(Vec::new()));
    add(&bus, "*.txt", trace_action(&trace, "tail"));
    let group = bus.create_group("files", false);
    bus.define(
        Attributes {
            group: Some(group),
            ..Attributes::default()
        },
        trace_action(&trace, "group"),
        &["User"],
        &[Selector::File("dir/*.txt".into())],
    )
    .unwrap();
    bus.define(
        Attributes::default(),
        trace_action(&trace, "buffer"),
        &["User"],
        &[Selector::Buffer(buffer)],
    )
    .unwrap();
    let occurrence = Occurrence::new(
        user(),
        Some(buffer),
        Names {
            file: "dir/foo.txt".into(),
            short: "dir/foo.txt".into(),
            full: "/root/dir/foo.txt".into(),
        },
    );
    assert_eq!(
        bus.dispatch(&cx, &occurrence, GroupFilter::All, Source::Explicit),
        3
    );
    assert_eq!(*trace.borrow(), ["tail", "group", "buffer"]);
    trace.borrow_mut().clear();
    bus.dispatch(
        &cx,
        &occurrence,
        GroupFilter::Group(group),
        Source::Explicit,
    );
    assert_eq!(*trace.borrow(), ["group"]);
    trace.borrow_mut().clear();
    let occurrence = Occurrence::new(user(), None, Names::same("foo.txt"));
    bus.dispatch(&cx, &occurrence, GroupFilter::All, Source::Explicit);
    assert_eq!(*trace.borrow(), ["tail"]);
}

#[test]
fn expanded_once_records_share_one_public_id_and_are_consumed_before_invocation() {
    let mut workspace = Workspace::new(Document::new(""));
    let buffer = workspace.current_id();
    let cx = Context::new(&mut workspace);
    let bus = Bus::new(false);
    let trace = Rc::new(RefCell::new(Vec::new()));
    let id = bus
        .define(
            Attributes {
                once: true,
                ..Attributes::default()
            },
            trace_action(&trace, "call"),
            &["User", "User"],
            &[Selector::File("*,*".into())],
        )
        .unwrap();
    let rows = bus.entries(&Filter::default());
    assert_eq!(rows.len(), 4);
    assert!(rows.iter().all(|row| row.id == Some(id)));
    assert_eq!(
        bus.dispatch(
            &cx,
            &event(buffer, "note"),
            GroupFilter::All,
            Source::Explicit
        ),
        4
    );
    assert_eq!(trace.borrow().len(), 4);
    assert!(bus.entries(&Filter::default()).is_empty());
}

#[test]
fn a_failed_later_pattern_keeps_only_the_registered_prefix() {
    let bus = Bus::new(false);
    let result = bus.define(
        Attributes::default(),
        Action::Command("set number".into()),
        &["User"],
        &[Selector::File("one,{,three".into())],
    );
    assert!(result.is_err());
    let rows = bus.entries(&Filter::default());
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].selector, Selector::File("one".into()));
}
