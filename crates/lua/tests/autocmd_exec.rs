use std::rc::Rc;
use v_ide_editor::{autocmd::Bus, document::Document, execution::Context, workspace::Workspace};
use v_ide_lua::host::PluginHost;

#[test]
fn native_exec_contract_preserves_copied_data_and_function_identity_across_nested_callbacks() {
    let bus = Rc::new(Bus::new(false));
    let host = PluginHost::new_with_bus(bus.clone()).unwrap();
    let mut ws = Workspace::new(Document::new(""));
    let cx = Context::with_observer(&mut ws, bus.as_ref());
    host.load_with_context(&cx, include_str!("autocmd/exec.lua"))
        .unwrap();
    assert!(cx.take_messages().is_empty());
}

#[test]
fn callback_errors_continue_and_invalid_later_events_preserve_the_executed_prefix() {
    let bus = Rc::new(Bus::new(false));
    let host = PluginHost::new_with_bus(bus.clone()).unwrap();
    let mut ws = Workspace::new(Document::new(""));
    let cx = Context::with_observer(&mut ws, bus.as_ref());
    host.load_with_context(
        &cx,
        r#"
      calls=0
      vibook.create_autocmd('User',{callback=function()error('expected callback error')end})
      vibook.create_autocmd('User',{callback=function()calls=calls+1;vibook.o.number=true end})
      assert(pcall(vibook.exec_autocmds,'User',{pattern='one',modeline=false}))
      assert(calls == 1 and vibook.o.number)
      assert(not pcall(vibook.exec_autocmds,{'User','Invalid'},{pattern='one',modeline=false}))
      assert(calls == 2)
    "#,
    )
    .unwrap();
    let messages = cx.take_messages();
    assert_eq!(messages.len(), 2);
    assert!(
        messages
            .iter()
            .all(|m| m.contains("expected callback error"))
    );
}

#[test]
fn exec_dispatch_and_retries_share_the_outer_conversion_budget() {
    let bus = Rc::new(Bus::new(false));
    let host = PluginHost::new_with_bus(bus.clone()).unwrap();
    let mut ws = Workspace::new(Document::new(""));
    let cx = Context::with_observer(&mut ws, bus.as_ref());
    host.load_with_context(
        &cx,
        r#"
      calls=0
      for i=1,2 do vibook.create_autocmd('User',{callback=function()calls=calls+1 end})end
      local data={};for i=1,350000 do data[i]=i end
      vibook.exec_autocmds('User',{data=data,modeline=false})
      assert(calls == 1)
      assert(not pcall(vibook.exec_autocmds,'User',{data=1,modeline=false}))
    "#,
    )
    .unwrap();
    let messages = cx.take_messages();
    assert_eq!(messages.len(), 1);
    assert!(messages[0].contains("budget"));
    host.load_with_context(
        &cx,
        "vibook.exec_autocmds('User',{data=1,modeline=false});assert(calls == 3)",
    )
    .unwrap();
    assert!(cx.take_messages().is_empty());
}
