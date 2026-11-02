use v_ide_editor::{
    document::Document,
    execution::Context,
    options::{Access, Name, Value},
    workspace::Workspace,
};
use v_ide_lua::host::PluginHost;

#[test]
fn entering_a_live_context_rejects_unflushed_legacy_writes_without_losing_them() {
    let host = PluginHost::new().unwrap();
    host.load("vibook.o.timeoutlen = 23").unwrap();
    let mut workspace = Workspace::new(Document::new(""));
    let cx = Context::new(&mut workspace);
    let error = host
        .load_with_context(&cx, "vibook.o.number = true")
        .unwrap_err();
    assert!(error.to_string().contains("pending option changes"));
    assert_eq!(
        cx.option(Name::Number, Access::Effective),
        Value::Bool(false)
    );
    assert_eq!(
        cx.option(Name::Timeoutlen, Access::Effective),
        Value::Number(1000)
    );
    let pending = host.take_option_changes();
    assert_eq!(pending.len(), 1);
    for (access, change) in pending {
        cx.set_option(access, &change).unwrap();
    }
    host.load_with_context(
        &cx,
        "assert(vibook.o.timeoutlen == 23); vibook.o.number = true",
    )
    .unwrap();
    assert_eq!(
        cx.option(Name::Number, Access::Effective),
        Value::Bool(true)
    );
    assert!(host.take_option_changes().is_empty());
}
