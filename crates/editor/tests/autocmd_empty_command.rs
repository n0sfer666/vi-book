use v_ide_editor::{
    autocmd::{
        Action, Attributes, Bus, EventName, GroupFilter, Names, Occurrence, Selector, Source,
    },
    document::Document,
    execution::Context,
    workspace::Workspace,
};

#[test]
fn an_empty_autocmd_command_is_a_valid_noop_without_a_diagnostic() {
    let mut workspace = Workspace::new(Document::new(""));
    let buffer = workspace.current_id();
    let cx = Context::new(&mut workspace);
    let bus = Bus::new(false);
    bus.define(
        Attributes::default(),
        Action::Command("".into()),
        &["User"],
        &[Selector::File("*".into())],
    )
    .unwrap();
    let occurrence = Occurrence::new(
        EventName::parse("User").unwrap(),
        Some(buffer),
        Names::same("Test"),
    );
    assert_eq!(
        bus.dispatch(&cx, &occurrence, GroupFilter::All, Source::Explicit),
        1
    );
    assert!(cx.take_messages().is_empty());
}
