use v_ide_editor::{
    document::Document,
    execution::{Context, OptionSet},
    workspace::Workspace,
};
use v_ide_lua::host::PluginHost;

#[test]
fn nested_commands_cannot_reset_the_outer_instruction_budget() {
    let host = PluginHost::new().unwrap();
    host.load("vibook.command('work', function() local n = 0; for i = 1,60000 do n = n+i end end)")
        .unwrap();
    let observer = |_: &Context<'_>, _: &OptionSet| {
        host.run_command("work")
            .map(|_| ())
            .map_err(|error| error.to_string())
    };
    let mut workspace = Workspace::new(Document::new(""));
    let cx = Context::with_observer(&mut workspace, &observer);
    let result = host.load_with_context(&cx, "for i = 1,600 do vibook.o.timeoutlen = i end");
    let messages = cx.take_messages();
    assert!(
        result.is_err()
            || messages
                .iter()
                .any(|message| message.contains("instruction budget"))
    );
    host.load_with_context(&cx, "assert(vibook.o.timeoutlen > 0)")
        .unwrap();
}
