#[path = "autocmd/bus_common.rs"]
mod common;

use common::{add, event};
use std::{cell::RefCell, rc::Rc};
use v_ide_editor::{
    autocmd::{
        Action, Attributes, Bus, EventName, Filter, GroupFilter, Invocation, Names, Occurrence,
        Selector, Source,
    },
    document::Document,
    execution::Context,
    options::{Access, Name, Value},
    workspace::Workspace,
};

#[test]
fn once_is_consumed_before_explicit_reentry_and_truthy_only_removes_the_current_record() {
    let mut workspace = Workspace::new(Document::new(""));
    let buffer = workspace.current_id();
    let cx = Context::new(&mut workspace);
    let bus = Rc::new(Bus::new(false));
    let trace = Rc::new(RefCell::new(Vec::new()));
    let calls = trace.clone();
    let weak = Rc::downgrade(&bus);
    bus.define(
        Attributes {
            once: true,
            ..Attributes::default()
        },
        Action::callback(move |cx: &Context<'_>, invocation: &Invocation<'_>| {
            let bus = weak.upgrade().unwrap();
            calls
                .borrow_mut()
                .push(bus.entries(&Filter::default()).len());
            if calls.borrow().len() == 1 {
                bus.dispatch(
                    cx,
                    invocation.occurrence,
                    GroupFilter::All,
                    Source::Explicit,
                );
            }
            Ok(false)
        }),
        &["User"],
        &[Selector::File("*,*".into())],
    )
    .unwrap();
    bus.dispatch(
        &cx,
        &event(buffer, "one"),
        GroupFilter::All,
        Source::Explicit,
    );
    assert_eq!(*trace.borrow(), [1, 0]);
    assert!(bus.entries(&Filter::default()).is_empty());
    let id = bus
        .define(
            Attributes::default(),
            Action::callback(|_: &Context<'_>, _: &Invocation<'_>| Ok(true)),
            &["User"],
            &[Selector::File("one,two".into())],
        )
        .unwrap();
    bus.dispatch(
        &cx,
        &event(buffer, "one"),
        GroupFilter::All,
        Source::Explicit,
    );
    let remaining = bus.entries(&Filter::default());
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].id, Some(id));
    assert_eq!(remaining[0].selector, Selector::File("two".into()));
}

#[test]
fn option_set_ex_handlers_are_inline_and_explicit_dispatch_inherits_option_values() {
    let mut workspace = Workspace::new(Document::new(""));
    let buffer = workspace.current_id();
    let bus = Rc::new(Bus::new(false));
    let trace = Rc::new(RefCell::new(Vec::new()));
    let calls = trace.clone();
    let weak = Rc::downgrade(&bus);
    bus.define(
        Attributes::default(),
        Action::callback(move |cx: &Context<'_>, invocation: &Invocation<'_>| {
            let bus = weak.upgrade().unwrap();
            let option = bus.option_values().unwrap();
            calls.borrow_mut().push((
                invocation.occurrence.buffer,
                option.old,
                invocation.occurrence.names.file.clone(),
            ));
            if calls.borrow().len() == 1 {
                cx.execute("set number").unwrap();
                let explicit = Occurrence::new(
                    EventName::parse("OptionSet").unwrap(),
                    Some(buffer),
                    Names::same("timeoutlen"),
                );
                bus.dispatch(cx, &explicit, GroupFilter::All, Source::Explicit);
            }
            Ok(false)
        }),
        &["OptionSet"],
        &[Selector::File("timeoutlen".into())],
    )
    .unwrap();
    let cx = Context::with_observer(&mut workspace, bus.as_ref());
    cx.execute("set timeoutlen=12").unwrap();
    assert_eq!(
        cx.option(Name::Number, Access::Effective),
        Value::Bool(true)
    );
    assert_eq!(
        *trace.borrow(),
        [
            (None, Value::Number(1000), String::new()),
            (Some(buffer), Value::Number(1000), "timeoutlen".into())
        ]
    );
    assert!(bus.option_values().is_none());
}

#[test]
fn case_policy_is_applied_before_freezing_matched_handles() {
    let mut workspace = Workspace::new(Document::new(""));
    let buffer = workspace.current_id();
    let cx = Context::new(&mut workspace);
    let bus = Bus::new(false);
    add(&bus, "É.txt", Action::Command("set number".into()));
    assert_eq!(
        bus.dispatch(
            &cx,
            &event(buffer, "é.TXT"),
            GroupFilter::All,
            Source::Explicit
        ),
        0
    );
    bus.set_ignore_case(true);
    assert_eq!(
        bus.dispatch(
            &cx,
            &event(buffer, "é.TXT"),
            GroupFilter::All,
            Source::Explicit
        ),
        1
    );
}
