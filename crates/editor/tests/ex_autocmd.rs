use std::rc::Rc;
use v_ide_editor::{
    autocmd::{Bus, Filter},
    document::Document,
    execution::Context,
    options::{Access, Name, Value},
    workspace::Workspace,
};

#[test]
fn ex_groups_definition_flags_clear_and_api_metadata_share_one_registry() {
    let bus = Rc::new(Bus::new(false));
    let mut ws = Workspace::new(Document::new(""));
    let cx = Context::with_observer(&mut ws, bus.as_ref());
    cx.execute("augroup Contract\nau User,BufEnter one,two ++once ++nested set number | set readonly\naugroup END").unwrap();
    let rows = bus.entries(&Filter::default());
    assert_eq!(rows.len(), 4);
    assert!(
        rows.iter()
            .all(|r| r.id.is_none() && r.attributes.once && r.attributes.nested)
    );
    assert_eq!(rows[0].action.command(), Some("set number | set readonly"));
    cx.execute("doautocmd <nomodeline> Contract User one")
        .unwrap();
    assert_eq!(cx.option(Name::Number, Access::Local), Value::Bool(true));
    assert_eq!(cx.option(Name::Readonly, Access::Local), Value::Bool(true));
    assert_eq!(bus.entries(&Filter::default()).len(), 3);
    cx.execute("augroup Contract\nau! User two set nonumber\naugroup END")
        .unwrap();
    cx.execute("do Contract User two").unwrap();
    assert_eq!(cx.option(Name::Number, Access::Local), Value::Bool(false));
    cx.execute("augroup Contract\nau!\naugroup END").unwrap();
    assert!(bus.entries(&Filter::default()).is_empty());
}

#[test]
fn invalid_ex_events_preserve_the_registry_and_doautocmd_stops_at_the_first_failed_record() {
    let bus = Bus::new(false);
    let mut ws = Workspace::new(Document::new(""));
    let cx = Context::with_observer(&mut ws, &bus);
    cx.execute("au User one set number").unwrap();
    assert!(cx.execute("au! User,Invalid * set readonly").is_err());
    assert_eq!(bus.entries(&Filter::default()).len(), 1);
    cx.execute("au User Err InvalidCommand\nau User Err set readonly")
        .unwrap();
    assert!(cx.execute("doautocmd User Err").is_err());
    assert_eq!(cx.option(Name::Readonly, Access::Local), Value::Bool(false));
    assert_eq!(cx.take_messages().len(), 1);
}

#[test]
fn ex_group_deletion_keeps_orphaned_records_and_cannot_delete_current_group() {
    let bus = Bus::new(false);
    let mut ws = Workspace::new(Document::new(""));
    let cx = Context::with_observer(&mut ws, &bus);
    cx.execute("aug Gone\nau User Marker set number").unwrap();
    assert!(cx.execute("augroup! Gone").is_err());
    cx.execute("aug END\naug! Gone").unwrap();
    assert!(bus.group("Gone").is_none());
    assert_eq!(bus.entries(&Filter::default()).len(), 1);
    cx.execute("do User Marker").unwrap();
    assert_eq!(cx.option(Name::Number, Access::Local), Value::Bool(true));
}

#[test]
fn ex_chain_stops_at_errors_and_keeps_comments_and_definition_bodies_separate() {
    let bus = Bus::new(false);
    let mut ws = Workspace::new(Document::new(""));
    let cx = Context::with_observer(&mut ws, &bus);
    cx.execute(":set number | set readonly \" ignore | set nonumber")
        .unwrap();
    assert_eq!(cx.option(Name::Number, Access::Local), Value::Bool(true));
    assert!(
        cx.execute("set nonumber | Invalid | set noreadonly")
            .is_err()
    );
    assert_eq!(cx.option(Name::Number, Access::Local), Value::Bool(false));
    assert_eq!(cx.option(Name::Readonly, Access::Local), Value::Bool(true));
    assert!(cx.execute("doautocmd *").is_err());
    assert!(cx.execute("doautocmd! User one").is_err());
}

#[test]
fn ex_option_handlers_abort_at_errors_while_direct_setters_keep_the_existing_continue_contract() {
    use v_ide_editor::autocmd::{Action, Attributes, Selector};
    use v_ide_editor::options::{Change, Operation};
    let bus = Bus::new(false);
    let mut ws = Workspace::new(Document::new(""));
    for command in ["InvalidCommand", "set readonly"] {
        bus.define(
            Attributes::default(),
            Action::Command(command.into()),
            &["OptionSet"],
            &[Selector::File("timeoutlen".into())],
        )
        .unwrap();
    }
    let cx = Context::with_observer(&mut ws, &bus);
    assert!(cx.execute("set timeoutlen=17").is_err());
    assert_eq!(cx.option(Name::Readonly, Access::Local), Value::Bool(false));
    cx.set_option(
        Access::Effective,
        &Change {
            name: Name::Timeoutlen,
            operation: Operation::Set(Value::Number(18)),
            text: "timeoutlen=18".into(),
        },
    )
    .unwrap();
    assert_eq!(cx.option(Name::Readonly, Access::Local), Value::Bool(true));
    assert_eq!(cx.take_messages().len(), 2);
}

#[test]
fn enew_reuses_only_clean_empty_unnamed_buffers_and_resets_buffer_options_without_notifications() {
    let bus = Bus::new(false);
    let mut ws = Workspace::new(Document::new(""));
    let cx = Context::with_observer(&mut ws, &bus);
    cx.execute("setglobal modeline\nsetlocal readonly nomodeline number")
        .unwrap();
    cx.execute("enew").unwrap();
    assert_eq!(cx.buffer_number(cx.buffer(0).unwrap()), Some(1));
    assert_eq!(cx.option(Name::Readonly, Access::Local), Value::Bool(false));
    assert_eq!(cx.option(Name::Modeline, Access::Local), Value::Bool(true));
    assert_eq!(cx.option(Name::Number, Access::Local), Value::Bool(true));
    cx.execute("file first.md\nset readonly nomodeline\nenew")
        .unwrap();
    assert_eq!(cx.buffer_number(cx.buffer(0).unwrap()), Some(2));
    assert_eq!(cx.option(Name::Readonly, Access::Local), Value::Bool(false));
    assert_eq!(cx.option(Name::Modeline, Access::Local), Value::Bool(false));
}

#[test]
fn long_runs_of_comment_lines_do_not_recurse_on_the_rust_stack() {
    let mut ws = Workspace::new(Document::new(""));
    let cx = Context::new(&mut ws);
    cx.execute(&format!("{}set number", "\" comment\n".repeat(100000)))
        .unwrap();
    assert_eq!(cx.option(Name::Number, Access::Local), Value::Bool(true));
}

#[test]
fn all_events_are_available_for_listing_and_clear_but_cannot_be_defined() {
    let bus = Bus::new(false);
    let mut ws = Workspace::new(Document::new(""));
    let cx = Context::with_observer(&mut ws, &bus);
    cx.execute("au User Marker set number").unwrap();
    assert!(cx.execute("autocmd! * Marker set readonly").is_err());
    let rows = bus.entries(&Filter::default());
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].action.command(), Some("set number"));
    cx.execute("autocmd! * Marker").unwrap();
    assert!(bus.entries(&Filter::default()).is_empty());
}
