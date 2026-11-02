use v_ide_editor::{
    document::Document,
    execution::{Context, OptionSet},
    options::{Access, Name, Value},
    workspace::Workspace,
};
use v_ide_lua::host::PluginHost;

#[test]
fn nested_lua_commands_share_the_live_context_and_finish_before_the_next_statement() {
    let host = PluginHost::new().unwrap();
    host.load("vibook.command('changed', function() vibook.wo.number = false end)")
        .unwrap();
    let observer = |_: &Context<'_>, _: &OptionSet| {
        host.run_command("changed")
            .map(|_| ())
            .map_err(|error| error.to_string())
    };
    let mut workspace = Workspace::new(Document::new(""));
    let cx = Context::with_observer(&mut workspace, &observer);
    host.load_with_context(&cx, "vibook.o.number = true; assert(vibook.o.number == false); vibook.opt.timeoutlen = 27; assert(vibook.o.timeoutlen == 27)").unwrap();
    assert_eq!(
        cx.option(Name::Number, Access::Effective),
        Value::Bool(false)
    );
    assert_eq!(
        cx.option(Name::Timeoutlen, Access::Effective),
        Value::Number(27)
    );
    assert!(host.take_option_changes().is_empty());
}

#[test]
fn stable_lua_accessors_work_across_scopes_and_option_objects_keep_captured_values() {
    let host = PluginHost::new().unwrap();
    let mut first = Workspace::new(Document::new(""));
    let cx = Context::new(&mut first);
    host.load_with_context(
        &cx,
        "saved_o = vibook.o; saved_opt = vibook.opt.timeoutlen; saved_o.timeoutlen = 31",
    )
    .unwrap();
    let mut second = Workspace::new(Document::new(""));
    let other = Context::new(&mut second);
    host.load_with_context(&other, "assert(saved_o.timeoutlen == 1000); assert(saved_opt:get() == 1000); saved_o.timeoutlen = 45").unwrap();
    host.load_with_context(
        &cx,
        "assert(saved_o.timeoutlen == 31); assert(saved_opt:get() == 1000)",
    )
    .unwrap();
    assert_eq!(
        other.option(Name::Timeoutlen, Access::Effective),
        Value::Number(45)
    );
}

#[test]
fn a_failed_scoped_script_preserves_writes_and_restores_the_snapshot_backend() {
    let host = PluginHost::new().unwrap();
    let mut workspace = Workspace::new(Document::new(""));
    let cx = Context::new(&mut workspace);
    assert!(
        host.load_with_context(&cx, "vibook.o.timeoutlen = 51; vibook.o.number = 'invalid'")
            .is_err()
    );
    assert_eq!(
        cx.option(Name::Timeoutlen, Access::Effective),
        Value::Number(51)
    );
    host.load("assert(vibook.o.timeoutlen == 51); vibook.o.number = true")
        .unwrap();
    assert_eq!(host.take_option_changes().len(), 1);
    assert_eq!(
        cx.option(Name::Number, Access::Effective),
        Value::Bool(false)
    );
}
