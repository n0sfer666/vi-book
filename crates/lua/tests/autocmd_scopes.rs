use std::rc::Rc;

use v_ide_editor::{autocmd::Bus, document::Document, execution::Context, workspace::Workspace};
use v_ide_lua::host::PluginHost;

#[test]
fn finalizers_never_observe_partly_installed_backends_and_gc_modes_survive_scopes() {
    let host = PluginHost::new_with_bus(Rc::new(Bus::new(false))).unwrap();
    let mut workspace = Workspace::new(Document::new(""));
    let cx = Context::new(&mut workspace);
    host.load(
        r#"
        finalizers = 0
        mismatches = 0
        live = false
        function garbage()
            collectgarbage('stop')
            for i=1,80 do
                local object = setmetatable({}, {__gc=function()
                    finalizers = finalizers + 1
                    local ok = pcall(vibook.get_autocmds, {buffer=0})
                    if ok ~= live then mismatches = mismatches + 1 end
                end})
            end
            collectgarbage('restart')
        end
        collectgarbage('incremental', 100, 1000, 1)
        garbage()
    "#,
    )
    .unwrap();
    for _ in 0..30 {
        host.load_with_context(
            &cx,
            r#"
            live = true
            collectgarbage('collect')
            assert(collectgarbage('isrunning'))
            garbage()
            live = false
        "#,
        )
        .unwrap();
    }
    host.load("collectgarbage('collect'); assert(finalizers == 2480); assert(mismatches == 0)")
        .unwrap();
    host.load_with_context(&cx, "collectgarbage('stop')")
        .unwrap();
    host.load("assert(not collectgarbage('isrunning'))")
        .unwrap();
    assert!(
        host.load_with_context(&cx, "collectgarbage('restart'); error('expected')")
            .is_err()
    );
    host.load("assert(collectgarbage('isrunning'))").unwrap();
}

#[test]
fn nested_native_callbacks_share_the_outer_instruction_budget() {
    let bus = Rc::new(Bus::new(false));
    let host = PluginHost::new_with_bus(bus.clone()).unwrap();
    let mut workspace = Workspace::new(Document::new(""));
    let cx = Context::with_observer(&mut workspace, bus.as_ref());
    host.load_with_context(
        &cx,
        r#"
        vibook.create_autocmd('OptionSet', {pattern='timeoutlen', callback=function()
            local sum = 0
            for i=1,60000 do sum = sum+i end
        end})
    "#,
    )
    .unwrap();
    let result = host.load_with_context(&cx, "for i=1,600 do vibook.o.timeoutlen = i end");
    assert!(
        result.is_err()
            || cx
                .take_messages()
                .iter()
                .any(|message| message.contains("instruction budget"))
    );
    host.load_with_context(&cx, "assert(vibook.o.timeoutlen > 0)")
        .unwrap();
}
