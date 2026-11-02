use std::cell::RefCell;

use v_ide_editor::{
    document::Document,
    execution::{Context, OptionSet},
    options::{Access, Change, Name, Operation, Value},
    workspace::Workspace,
};

fn change(name: Name, value: Value) -> Change {
    Change {
        name,
        operation: Operation::Set(value),
        text: name.as_str().into(),
    }
}

#[test]
fn option_observers_run_inline_and_can_execute_ex_without_borrowing_the_workspace() {
    let mut ws = Workspace::new(Document::new(""));
    let trace = RefCell::new(Vec::new());
    let observer = |cx: &Context<'_>, event: &OptionSet| {
        trace.borrow_mut().push(event.name);
        assert_eq!(cx.option(event.name, Access::Effective), event.new);
        cx.execute("set nonumber").unwrap();
        Ok(())
    };
    let cx = Context::with_observer(&mut ws, &observer);
    cx.execute("set number").unwrap();
    assert_eq!(
        cx.option(Name::Number, Access::Effective),
        Value::Bool(false)
    );
    assert_eq!(*trace.borrow(), [Name::Number]);
    cx.execute("set timeoutlen=12").unwrap();
    assert_eq!(*trace.borrow(), [Name::Number, Name::Timeoutlen]);
}

#[test]
fn native_option_event_metadata_distinguishes_set_local_and_global_in_all_scopes() {
    for name in [Name::Timeoutlen, Name::Readonly, Name::Number] {
        for access in [Access::Effective, Access::Local, Access::Global] {
            let mut ws = Workspace::new(Document::new(""));
            let trace = RefCell::new(Vec::new());
            let observer = |_: &Context<'_>, event: &OptionSet| {
                trace.borrow_mut().push(event.clone());
                Ok(())
            };
            let cx = Context::with_observer(&mut ws, &observer);
            let new = if name == Name::Timeoutlen {
                Value::Number(12)
            } else {
                Value::Bool(true)
            };
            cx.set_option(access, &change(name, new)).unwrap();
            let rows = trace.borrow();
            assert_eq!(rows.len(), 1);
            let event = &rows[0];
            assert_eq!(event.name, name);
            assert_eq!(event.old, name.default_value());
            assert_eq!(event.new, new);
            assert_eq!(
                event.old_local,
                (access != Access::Global).then_some(name.default_value())
            );
            assert_eq!(
                event.old_global,
                (access != Access::Local).then_some(name.default_value())
            );
            assert_eq!(event.access, access);
        }
    }
}

#[test]
fn same_value_writes_notify_but_queries_invalid_writes_and_all_reset_do_not() {
    let mut ws = Workspace::new(Document::new(""));
    let trace = RefCell::new(Vec::new());
    let observer = |_: &Context<'_>, event: &OptionSet| {
        trace.borrow_mut().push(event.clone());
        Ok(())
    };
    let cx = Context::with_observer(&mut ws, &observer);
    cx.execute("set number number").unwrap();
    assert_eq!(trace.borrow().len(), 2);
    assert_eq!(trace.borrow()[1].old, Value::Bool(true));
    assert!(cx.execute("set timeoutlen=-1").is_err());
    cx.execute("set number? all&").unwrap();
    assert_eq!(trace.borrow().len(), 2);
    assert_eq!(
        cx.option(Name::Number, Access::Effective),
        Value::Bool(false)
    );
    assert_eq!(cx.take_messages(), ["  number"]);
}

#[test]
fn callback_errors_keep_applied_writes_and_are_reported_for_ex_but_not_lua_style_setters() {
    let mut ws = Workspace::new(Document::new(""));
    let observer = |_: &Context<'_>, _: &OptionSet| Err("option boom".into());
    let cx = Context::with_observer(&mut ws, &observer);
    assert_eq!(
        cx.set_option(
            Access::Effective,
            &change(Name::Timeoutlen, Value::Number(13))
        )
        .unwrap(),
        Some(Value::Number(13))
    );
    assert_eq!(
        cx.option(Name::Timeoutlen, Access::Effective),
        Value::Number(13)
    );
    assert_eq!(cx.take_messages(), ["option boom"]);
    assert!(cx.execute("set timeoutlen=14").is_err());
    assert_eq!(
        cx.option(Name::Timeoutlen, Access::Effective),
        Value::Number(14)
    );
    assert_eq!(cx.take_messages(), ["option boom"]);
}

#[test]
fn ex_keeps_a_valid_prefix_and_pending_query_output_when_later_input_is_invalid() {
    let mut ws = Workspace::new(Document::new(""));
    let cx = Context::new(&mut ws);
    assert!(
        cx.execute("set timeoutlen=23 timeoutlen? badoption number")
            .is_err()
    );
    assert_eq!(
        cx.option(Name::Timeoutlen, Access::Effective),
        Value::Number(23)
    );
    assert_eq!(
        cx.option(Name::Number, Access::Effective),
        Value::Bool(false)
    );
    assert_eq!(cx.take_messages(), ["  timeoutlen=23"]);
}
