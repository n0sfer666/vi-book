#[path = "autocmd/bus_common.rs"]
mod common;

use common::{add, event};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use v_ide_editor::{
    autocmd::{Action, Bus, Filter, GroupFilter, Invocation, Source},
    document::Document,
    execution::Context,
    options::{Access, Name, Value},
    workspace::Workspace,
};

#[test]
fn matched_handles_are_frozen_and_deleted_records_are_skipped() {
    let mut workspace = Workspace::new(Document::new(""));
    let buffer = workspace.current_id();
    let cx = Context::new(&mut workspace);
    let bus = Rc::new(Bus::new(false));
    let trace = Rc::new(RefCell::new(Vec::new()));
    let later = Rc::new(Cell::new(None));
    let weak = Rc::downgrade(&bus);
    let deleted = later.clone();
    let calls = trace.clone();
    add(
        &bus,
        "*",
        Action::callback(move |_: &Context<'_>, _: &Invocation<'_>| {
            calls.borrow_mut().push("first");
            let bus = weak.upgrade().unwrap();
            bus.delete(deleted.take().unwrap()).unwrap();
            let added = calls.clone();
            add(
                &bus,
                "*",
                Action::callback(move |_: &Context<'_>, _: &Invocation<'_>| {
                    added.borrow_mut().push("new");
                    Ok(false)
                }),
            );
            Ok(true)
        }),
    );
    let calls = trace.clone();
    later.set(Some(add(
        &bus,
        "*",
        Action::callback(move |_: &Context<'_>, _: &Invocation<'_>| {
            calls.borrow_mut().push("old");
            Ok(false)
        }),
    )));
    bus.dispatch(
        &cx,
        &event(buffer, "note"),
        GroupFilter::All,
        Source::Explicit,
    );
    assert_eq!(*trace.borrow(), ["first"]);
    bus.dispatch(
        &cx,
        &event(buffer, "note"),
        GroupFilter::All,
        Source::Explicit,
    );
    assert_eq!(*trace.borrow(), ["first", "new"]);
}

#[test]
fn errors_continue_other_records_and_ex_actions_execute_inline() {
    let mut workspace = Workspace::new(Document::new(""));
    let buffer = workspace.current_id();
    let cx = Context::new(&mut workspace);
    let bus = Bus::new(false);
    add(
        &bus,
        "*",
        Action::callback(|_: &Context<'_>, _: &Invocation<'_>| Err("bad callback".into())),
    );
    add(&bus, "*", Action::Command("set number".into()));
    assert_eq!(
        bus.dispatch(
            &cx,
            &event(buffer, "note"),
            GroupFilter::All,
            Source::Explicit
        ),
        2
    );
    assert_eq!(
        cx.option(Name::Number, Access::Effective),
        Value::Bool(true)
    );
    assert_eq!(cx.take_messages(), ["bad callback"]);
    assert_eq!(bus.entries(&Filter::default()).len(), 2);
}
