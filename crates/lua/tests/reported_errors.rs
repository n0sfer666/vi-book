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

#[test]
fn a_saved_error_is_reported_again_in_a_new_context() {
    let bus = Rc::new(Bus::new(false));
    let host = PluginHost::new_with_bus(bus.clone()).unwrap();
    let mut ws = Workspace::new(Document::new("seed"));
    {
        let cx = Context::with_observer(&mut ws, bus.as_ref());
        host.load_with_context(
            &cx,
            r#"
    vibook.create_autocmd('User',{pattern='Inner',callback=function()error('saved marker')end})
    local ok;ok,saved_error=pcall(vibook.cmd,'doautocmd <nomodeline> User Inner');assert(not ok)
    vibook.create_autocmd('User',{pattern='Later',callback=function()error(saved_error)end})
  "#,
        )
        .unwrap();
        assert_eq!(cx.take_messages().len(), 1);
    }
    let cx = Context::with_observer(&mut ws, bus.as_ref());
    assert!(
        host.load_with_context(&cx, "vibook.cmd('doautocmd <nomodeline> User Later')")
            .is_err()
    );
    let messages = cx.take_messages();
    assert_eq!(messages.len(), 1, "{messages:?}");
    assert!(messages[0].contains("saved marker"));
}

#[test]
fn a_saved_error_is_reported_again_in_a_later_call_on_the_same_context() {
    let bus = Rc::new(Bus::new(false));
    let host = PluginHost::new_with_bus(bus.clone()).unwrap();
    let mut ws = Workspace::new(Document::new("seed"));
    let cx = Context::with_observer(&mut ws, bus.as_ref());
    host.load_with_context(
        &cx,
        r#"
   vibook.create_autocmd('User',{pattern='Inner',callback=function()error('saved marker')end})
   local ok;ok,saved_error=pcall(vibook.cmd,'doautocmd <nomodeline> User Inner');assert(not ok)
   vibook.create_autocmd('User',{pattern='Later',callback=function()error(saved_error)end})
 "#,
    )
    .unwrap();
    assert_eq!(cx.take_messages().len(), 1);
    assert!(
        host.load_with_context(&cx, "vibook.cmd('doautocmd <nomodeline> User Later')")
            .is_err()
    );
    assert_eq!(cx.take_messages().len(), 1);
}

#[test]
fn an_unrelated_new_diagnostic_does_not_hide_a_saved_error() {
    let bus = Rc::new(Bus::new(false));
    let host = PluginHost::new_with_bus(bus.clone()).unwrap();
    let mut ws = Workspace::new(Document::new("seed"));
    let cx = Context::with_observer(&mut ws, bus.as_ref());
    host.load_with_context(
        &cx,
        r#"
   vibook.create_autocmd('User',{pattern='Inner',callback=function()error('saved marker')end})
   local ok;ok,saved_error=pcall(vibook.cmd,'doautocmd <nomodeline> User Inner');assert(not ok)
   vibook.create_autocmd('User',{pattern='Later',callback=function()
      vibook.exec_autocmds('User',{pattern='Inner',modeline=false});error(saved_error)
   end})
 "#,
    )
    .unwrap();
    cx.take_messages();
    assert!(
        host.load_with_context(&cx, "vibook.cmd('doautocmd <nomodeline> User Later')")
            .is_err()
    );
    let messages = cx.take_messages();
    assert_eq!(messages.len(), 2, "{messages:?}");
}
