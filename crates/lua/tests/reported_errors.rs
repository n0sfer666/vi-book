use std::rc::Rc;
use v_ide_editor::{autocmd::Bus, document::Document, execution::Context, workspace::Workspace};
use v_ide_lua::host::PluginHost;

#[test]
fn nested_lua_ex_failures_keep_one_diagnostic_across_callback_boundaries() {
    let bus = Rc::new(Bus::new(false));
    let host = PluginHost::new_with_bus(bus.clone()).unwrap();
    let mut ws = Workspace::new(Document::new("seed"));
    let cx = Context::with_observer(&mut ws, bus.as_ref());
    let result=host.load_with_context(&cx,r#"
   vibook.create_autocmd('User',{pattern='Inner',callback=function()error('nested marker')end})
   vibook.create_autocmd('User',{pattern='Outer',callback=function()vibook.cmd('doautocmd <nomodeline> User Inner')end})
   vibook.cmd('doautocmd <nomodeline> User Outer')
 "#);
    assert!(result.is_err());
    let messages = cx.take_messages();
    assert_eq!(messages.len(), 1, "{messages:?}");
    assert!(messages[0].contains("nested marker"));
    host.load_with_context(&cx, "vibook.cmd('set number')")
        .unwrap();
}

#[test]
fn an_independent_lua_error_after_an_api_diagnostic_remains_an_error() {
    let bus = Rc::new(Bus::new(false));
    let host = PluginHost::new_with_bus(bus.clone()).unwrap();
    let mut ws = Workspace::new(Document::new("seed"));
    let cx = Context::with_observer(&mut ws, bus.as_ref());
    let error = host
        .load_with_context(
            &cx,
            r#"
   vibook.create_autocmd('User',{pattern='Inner',callback=function()error('inner marker')end})
   vibook.exec_autocmds('User',{pattern='Inner',modeline=false})
   error('independent marker')
 "#,
        )
        .unwrap_err();
    assert!(error.to_string().contains("independent marker"));
    let messages = cx.take_messages();
    assert_eq!(messages.len(), 1);
    assert!(messages[0].contains("inner marker"));
}
