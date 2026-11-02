use std::rc::Rc;

use v_ide_editor::{
    autocmd::{Bus, EventName, GroupFilter, Names, Occurrence, Source},
    document::Document,
    execution::Context,
    workspace::Workspace,
};
use v_ide_lua::host::PluginHost;

#[test]
fn native_definition_group_and_filter_contract() {
    let bus = Rc::new(Bus::new(false));
    let host = PluginHost::new_with_bus(bus).unwrap();
    host.load(include_str!("autocmd/definitions.lua")).unwrap();
}

#[test]
fn buffer_zero_is_resolved_in_each_live_context_and_wrappers_stay_valid() {
    let bus = Rc::new(Bus::new(false));
    let host = PluginHost::new_with_bus(bus).unwrap();
    let mut workspace = Workspace::new(Document::new(""));
    let cx = Context::new(&mut workspace);
    host.load_with_context(
        &cx,
        r#"
        saved_create = vibook.create_autocmd
        saved_get = vibook.get_autocmds
        local id = saved_create('User', {buffer=0, command=''})
        local row = saved_get({id=id})[1]
        assert(row.buflocal and row.buffer == 1 and row.pattern == '<buffer=1>')
        assert(#saved_get({buffer={0,1}, event='User'}) == 1)
        assert(not pcall(saved_create, 'User', {buffer=-1, command=''}))
        assert(not pcall(saved_create, 'User', {buffer=1.5, command=''}))
        assert(not pcall(saved_get, {buffer=2000}))
        assert(not pcall(saved_get, {buffer=0, pattern='*'}))
    "#,
    )
    .unwrap();
    host.load("assert(type(saved_get) == 'function' and type(saved_create) == 'function'); assert(not pcall(saved_get, {event='User'}))").unwrap();
    host.load_with_context(
        &cx,
        "vibook.clear_autocmds({buffer=0}); assert(#saved_get({event='User'}) == 0)",
    )
    .unwrap();
}

#[test]
fn callbacks_are_synchronous_can_mutate_registry_and_truthy_removes_only_current_record() {
    let bus = Rc::new(Bus::new(false));
    let host = PluginHost::new_with_bus(bus.clone()).unwrap();
    let mut workspace = Workspace::new(Document::new(""));
    let cx = Context::with_observer(&mut workspace, bus.as_ref());
    host.load_with_context(
        &cx,
        r#"
        calls = 0
        local cb = function(a)
            calls = calls + 1
            assert(a.event == 'User' and a.match == 'one' and a.file == 'one')
            assert(a.buf == 1 and a.group == nil and a.data == nil)
            assert(a.id == id)
            vibook.o.number = true
            vibook.create_autocmd('User', {pattern='later', command='set nonumber'})
            return 0
        end
        id = vibook.create_autocmd('User', {pattern='one,two', callback=cb})
        assert(vibook.get_autocmds({id=id})[1].callback == cb)
    "#,
    )
    .unwrap();
    let occurrence = Occurrence::new(
        EventName::parse("User").unwrap(),
        cx.buffer(0),
        Names::same("one"),
    );
    assert_eq!(
        bus.dispatch(&cx, &occurrence, GroupFilter::All, Source::Explicit),
        1
    );
    host.load_with_context(&cx, "assert(calls == 1 and vibook.o.number); local rows=vibook.get_autocmds({id=id}); assert(#rows == 1 and rows[1].pattern == 'two')").unwrap();
    assert!(cx.take_messages().is_empty());
}

#[test]
fn option_callbacks_finish_inline_and_named_callbacks_ignore_truthy_results() {
    let bus = Rc::new(Bus::new(false));
    let host = PluginHost::new_with_bus(bus.clone()).unwrap();
    let mut workspace = Workspace::new(Document::new(""));
    let cx = Context::with_observer(&mut workspace, bus.as_ref());
    host.load_with_context(&cx, r#"
        function Named(...)
            assert(select('#', ...) == 0)
            vibook.wo.number = false
            return true
        end
        id = vibook.create_autocmd('OptionSet', {pattern='timeoutlen', callback='v:lua.Named'})
        vibook.o.number = true
        vibook.o.timeoutlen = 25
        assert(not vibook.o.number)
        assert(#vibook.get_autocmds({id=id}) == 1)
        vibook.create_autocmd('OptionSet', {pattern='timeoutlen', callback=function() error('callback failed') end})
        vibook.o.timeoutlen = 26
        assert(vibook.o.timeoutlen == 26)
    "#).unwrap();
    let messages = cx.take_messages();
    assert_eq!(messages.len(), 1);
    assert!(messages[0].contains("callback failed"));
    assert!(host.take_option_changes().is_empty());
}

#[test]
fn bus_and_vm_callbacks_do_not_form_a_strong_ownership_cycle() {
    let bus = Rc::new(Bus::new(false));
    let weak = Rc::downgrade(&bus);
    let host = PluginHost::new_with_bus(bus.clone()).unwrap();
    host.load("vibook.create_autocmd('User', {callback=function() end})")
        .unwrap();
    drop(bus);
    assert!(weak.upgrade().is_some());
    drop(host);
    assert!(weak.upgrade().is_none());
}

#[test]
fn native_definition_prefixes_empty_filters_and_numeric_booleans() {
    let host = PluginHost::new_with_bus(Rc::new(Bus::new(false))).unwrap();
    host.load(include_str!("autocmd/definition_edges.lua"))
        .unwrap();
}

#[test]
fn native_buffer_patterns_and_failed_prefix_are_resolved_through_context() {
    let host = PluginHost::new_with_bus(Rc::new(Bus::new(false))).unwrap();
    let mut workspace = Workspace::new(Document::new(""));
    let cx = Context::new(&mut workspace);
    host.load_with_context(&cx, include_str!("autocmd/buffer_patterns.lua"))
        .unwrap();
}

#[test]
fn dictionaries_and_extra_arguments_are_rejected_before_mutating_the_registry() {
    let host = PluginHost::new_with_bus(Rc::new(Bus::new(false))).unwrap();
    host.load(include_str!("autocmd/validation.lua")).unwrap();
}
