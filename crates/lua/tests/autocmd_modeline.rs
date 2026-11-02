use std::rc::Rc;
use v_ide_editor::{
    autocmd::Bus,
    document::Document,
    execution::Context,
    options::{Access, Name, Value},
    workspace::Workspace,
};
use v_ide_lua::host::PluginHost;

#[test]
fn modelines_run_once_after_all_callbacks_and_respect_native_gates() {
    let bus = Rc::new(Bus::new(false));
    let host = PluginHost::new_with_bus(bus.clone()).unwrap();
    let mut ws = Workspace::new(Document::new("vim: set number readonly:"));
    let cx = Context::with_observer(&mut ws, bus.as_ref());
    host.load_with_context(&cx, include_str!("autocmd/exec_modeline.lua"))
        .unwrap();
    assert!(cx.take_messages().is_empty());
}

#[test]
fn modeline_errors_are_diagnostics_preserving_the_valid_prefix_and_api_success() {
    let bus = Rc::new(Bus::new(false));
    let host = PluginHost::new_with_bus(bus.clone()).unwrap();
    let mut ws = Workspace::new(Document::new(
        "vim: set number unknown_xyz readonly:\nvim: set timeoutlen=29:",
    ));
    let cx = Context::with_observer(&mut ws, bus.as_ref());
    host.load_with_context(
        &cx,
        r#"
      vibook.o.modeline=true
      vibook.create_autocmd('User',{command=''})
      assert(pcall(vibook.exec_autocmds,'User',{}))
      assert(vibook.wo.number and not vibook.bo.readonly and vibook.o.timeoutlen == 1000)
    "#,
    )
    .unwrap();
    let messages = cx.take_messages();
    assert_eq!(messages.len(), 1);
    assert!(messages[0].contains("E518"));
}

#[test]
fn hidden_buffer_execution_keeps_current_context_and_reads_current_modelines() {
    let bus = Rc::new(Bus::new(false));
    let host = PluginHost::new_with_bus(bus.clone()).unwrap();
    let mut ws = Workspace::new(Document::new("vim: set number:"));
    let mut hidden = Document::new("vim: set readonly:");
    hidden.name = "target/hidden.md".into();
    let hidden = ws.add(hidden);
    let window = ws.active_window();
    {
        let cx = Context::with_observer(&mut ws, bus.as_ref());
        host.load_with_context(
            &cx,
            r#"
        vibook.o.modeline=true
        local calls=0
        vibook.create_autocmd('User',{buffer=2,callback=function(a)
          assert(a.buf == 2 and a.file:sub(-17) == '/target/hidden.md' and a.file == a.match)
          assert(not vibook.wo.number and not vibook.bo.readonly)
          calls=calls+1
        end})
        vibook.exec_autocmds('User',{buf=2})
        assert(calls == 1 and vibook.wo.number and not vibook.bo.readonly)
      "#,
        )
        .unwrap();
        assert!(cx.take_messages().is_empty());
    }
    assert_eq!(
        ws.option_at(hidden, window, Name::Readonly, Access::Local),
        Some(Value::Bool(false))
    );
}

#[test]
fn modelines_do_not_reenter_when_option_callbacks_explicitly_dispatch_an_event() {
    let bus = Rc::new(Bus::new(false));
    let host = PluginHost::new_with_bus(bus.clone()).unwrap();
    let mut ws = Workspace::new(Document::new("vim: set number:"));
    let cx = Context::with_observer(&mut ws, bus.as_ref());
    host.load_with_context(
        &cx,
        r#"
      vibook.o.modeline=true
      local calls=0
      vibook.create_autocmd('User',{callback=function()calls=calls+1 end})
      vibook.create_autocmd('OptionSet',{pattern='number',callback=function()
        vibook.exec_autocmds('User',{})
      end})
      vibook.exec_autocmds('User',{})
      assert(calls == 2)
    "#,
    )
    .unwrap();
    assert!(cx.take_messages().is_empty());
}

#[test]
fn file_event_callback_matches_are_absolute_while_other_matches_stay_literal() {
    let bus = Rc::new(Bus::new(false));
    let host = PluginHost::new_with_bus(bus.clone()).unwrap();
    let mut ws = Workspace::new(Document::new(""));
    let cx = Context::with_observer(&mut ws, bus.as_ref());
    host.load_with_context(&cx, include_str!("autocmd/exec_names.lua"))
        .unwrap();
    assert!(cx.take_messages().is_empty());
}
