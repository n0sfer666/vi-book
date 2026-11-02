use std::rc::Rc;

use v_ide_editor::{
    autocmd::{Bus, EventName, GroupFilter, Names, Occurrence, Source},
    document::Document,
    execution::Context,
    workspace::Workspace,
};
use v_ide_lua::host::PluginHost;

#[test]
fn abuf_targets_the_event_buffer_instead_of_the_current_buffer() {
    let bus = Rc::new(Bus::new(false));
    let host = PluginHost::new_with_bus(bus.clone()).unwrap();
    let mut workspace = Workspace::new(Document::new(""));
    let hidden = workspace.add(Document::new("hidden"));
    let cx = Context::new(&mut workspace);
    host.load("target_buffer=2").unwrap();
    host.load_with_context(&cx, include_str!("autocmd/active_buffer.lua"))
        .unwrap();
    let occurrence = Occurrence::new(
        EventName::parse("User").unwrap(),
        Some(hidden),
        Names::same("ActiveBuffer"),
    );
    bus.dispatch(&cx, &occurrence, GroupFilter::All, Source::Explicit);
    assert!(cx.take_messages().is_empty());
    host.load("assert(active_buffer_result == 2)").unwrap();
}

#[test]
fn named_callbacks_receive_no_arguments_and_resolve_metatable_backed_globals() {
    let bus = Rc::new(Bus::new(false));
    let host = PluginHost::new_with_bus(bus.clone()).unwrap();
    let mut workspace = Workspace::new(Document::new(""));
    let cx = Context::new(&mut workspace);
    host.load(
        r#"
        calls=0
        Proxy=setmetatable({}, {__index=function(_, key)
            assert(key == 'Callback')
            return function(...)
                assert(select('#',...) == 0)
                calls=calls+1
                return true
            end
        end})
        named=vibook.create_autocmd('User', {callback='v:lua.Proxy.Callback'})
    "#,
    )
    .unwrap();
    let occurrence = Occurrence::new(
        EventName::parse("User").unwrap(),
        cx.buffer(0),
        Names::same("sample"),
    );
    bus.dispatch(&cx, &occurrence, GroupFilter::All, Source::Explicit);
    bus.dispatch(&cx, &occurrence, GroupFilter::All, Source::Explicit);
    assert!(cx.take_messages().is_empty());
    host.load("assert(calls == 2 and #vibook.get_autocmds({id=named}) == 1)")
        .unwrap();
}

#[test]
fn named_callbacks_accept_lua_callable_tables_and_keep_the_record() {
    let bus = Rc::new(Bus::new(false));
    let host = PluginHost::new_with_bus(bus.clone()).unwrap();
    let mut workspace = Workspace::new(Document::new(""));
    let cx = Context::new(&mut workspace);
    host.load(include_str!("autocmd/named_callables.lua"))
        .unwrap();
    let occurrence = Occurrence::new(
        EventName::parse("User").unwrap(),
        cx.buffer(0),
        Names::same("NamedCallable"),
    );
    bus.dispatch(&cx, &occurrence, GroupFilter::All, Source::Explicit);
    bus.dispatch(&cx, &occurrence, GroupFilter::All, Source::Explicit);
    assert!(cx.take_messages().is_empty());
    host.load(
        "assert(callable_calls == 2 and #vibook.get_autocmds({group='NamedCallables'}) == 1)",
    )
    .unwrap();
}
