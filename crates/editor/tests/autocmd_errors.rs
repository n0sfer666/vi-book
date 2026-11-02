use std::rc::Rc;
use v_ide_editor::{
    autocmd::{
        Action, Attributes, Bus, EventName, GroupFilter, Invocation, Names, Occurrence, Selector,
        Source,
    },
    document::Document,
    execution::Context,
    workspace::Workspace,
};

#[test]
fn nested_ex_boundaries_do_not_report_the_same_autocmd_error_again() {
    let mut workspace = Workspace::new(Document::new(""));
    let buffer = workspace.current_id();
    let bus = Bus::new(false);
    bus.define(
        Attributes::default(),
        Action::Command("set badoption".into()),
        &["OptionSet"],
        &[Selector::File("timeoutlen".into())],
    )
    .unwrap();
    bus.define(
        Attributes {
            nested: true,
            ..Attributes::default()
        },
        Action::Command("set timeoutlen=22".into()),
        &["User"],
        &[Selector::File("Go".into())],
    )
    .unwrap();
    let cx = Context::with_observer(&mut workspace, &bus);
    let event = Occurrence::new(
        EventName::parse("User").unwrap(),
        Some(buffer),
        Names::same("Go"),
    );
    bus.dispatch(&cx, &event, GroupFilter::All, Source::Explicit);
    let messages = cx.take_messages();
    assert_eq!(messages.len(), 1, "{messages:?}");
    assert!(messages[0].contains("E518"));
}

#[test]
fn a_separate_callback_error_is_not_hidden_by_an_earlier_nested_diagnostic() {
    let mut workspace = Workspace::new(Document::new(""));
    let buffer = workspace.current_id();
    let bus = Rc::new(Bus::new(false));
    bus.define(
        Attributes::default(),
        Action::Command("set badoption".into()),
        &["User"],
        &[Selector::File("Inner".into())],
    )
    .unwrap();
    let weak = Rc::downgrade(&bus);
    bus.define(
        Attributes::default(),
        Action::callback(move |cx: &Context<'_>, _: &Invocation<'_>| {
            let event = Occurrence::new(
                EventName::parse("User").unwrap(),
                Some(buffer),
                Names::same("Inner"),
            );
            weak.upgrade()
                .unwrap()
                .dispatch(cx, &event, GroupFilter::All, Source::Explicit);
            Err("outer error".into())
        }),
        &["User"],
        &[Selector::File("Outer".into())],
    )
    .unwrap();
    let cx = Context::with_observer(&mut workspace, bus.as_ref());
    let event = Occurrence::new(
        EventName::parse("User").unwrap(),
        Some(buffer),
        Names::same("Outer"),
    );
    bus.dispatch(&cx, &event, GroupFilter::All, Source::Explicit);
    let messages = cx.take_messages();
    assert_eq!(messages.len(), 2);
    assert!(messages[0].contains("E518"));
    assert_eq!(messages[1], "outer error");
}
