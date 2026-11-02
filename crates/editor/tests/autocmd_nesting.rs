#[path = "autocmd/bus_common.rs"]
mod common;

use common::{add, event};
use std::{cell::Cell, rc::Rc};
use v_ide_editor::{
    autocmd::{Action, Attributes, Bus, GroupFilter, Invocation, Selector, Source},
    document::Document,
    execution::Context,
    workspace::Workspace,
};

#[test]
fn automatic_nested_events_obey_nested_while_explicit_events_bypass_it() {
    for nested in [false, true] {
        let mut workspace = Workspace::new(Document::new(""));
        let buffer = workspace.current_id();
        let cx = Context::new(&mut workspace);
        let bus = Rc::new(Bus::new(false));
        let count = Rc::new(Cell::new(0));
        let calls = count.clone();
        add(
            &bus,
            "Inner",
            Action::callback(move |_: &Context<'_>, _: &Invocation<'_>| {
                calls.set(calls.get() + 1);
                Ok(false)
            }),
        );
        let weak = Rc::downgrade(&bus);
        bus.define(
            Attributes {
                nested,
                ..Attributes::default()
            },
            Action::callback(move |cx: &Context<'_>, _: &Invocation<'_>| {
                let bus = weak.upgrade().unwrap();
                bus.dispatch(
                    cx,
                    &event(buffer, "Inner"),
                    GroupFilter::All,
                    Source::Automatic,
                );
                bus.dispatch(
                    cx,
                    &event(buffer, "Inner"),
                    GroupFilter::All,
                    Source::Explicit,
                );
                Ok(false)
            }),
            &["User"],
            &[Selector::File("Outer".into())],
        )
        .unwrap();
        bus.dispatch(
            &cx,
            &event(buffer, "Outer"),
            GroupFilter::All,
            Source::Explicit,
        );
        assert_eq!(count.get(), if nested { 2 } else { 1 });
        bus.dispatch(
            &cx,
            &event(buffer, "Inner"),
            GroupFilter::All,
            Source::Automatic,
        );
        assert_eq!(count.get(), if nested { 3 } else { 2 });
    }
}

#[test]
fn native_nesting_limit_diagnoses_at_ten_and_restores_dispatch_state() {
    let mut workspace = Workspace::new(Document::new(""));
    let buffer = workspace.current_id();
    let cx = Context::new(&mut workspace);
    let bus = Rc::new(Bus::new(false));
    let count = Rc::new(Cell::new(0));
    let calls = count.clone();
    let weak = Rc::downgrade(&bus);
    add(
        &bus,
        "*",
        Action::callback(move |cx: &Context<'_>, invocation: &Invocation<'_>| {
            calls.set(calls.get() + 1);
            if calls.get() < 20 {
                weak.upgrade().unwrap().dispatch(
                    cx,
                    invocation.occurrence,
                    GroupFilter::All,
                    Source::Explicit,
                );
            }
            Ok(false)
        }),
    );
    bus.dispatch(
        &cx,
        &event(buffer, "note"),
        GroupFilter::All,
        Source::Explicit,
    );
    assert_eq!(count.get(), 10);
    assert!(
        cx.take_messages()
            .iter()
            .any(|message| message.contains("E218"))
    );
    bus.dispatch(
        &cx,
        &event(buffer, "note"),
        GroupFilter::All,
        Source::Explicit,
    );
    assert_eq!(count.get(), 20);
    assert!(cx.take_messages().is_empty());
}
