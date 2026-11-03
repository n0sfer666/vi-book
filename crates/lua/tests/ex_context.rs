use std::rc::Rc;
use v_ide_editor::{autocmd::Bus, document::Document, execution::Context, workspace::Workspace};
use v_ide_lua::host::PluginHost;

#[test]
fn native_ex_and_lua_callbacks_share_live_buffers_and_finish_before_continuation() {
    let bus = Rc::new(Bus::new(false));
    let host = PluginHost::new_with_bus(bus.clone()).unwrap();
    let mut doc = Document::new("seed");
    doc.name = "target/first.md".into();
    let mut ws = Workspace::new(doc);
    let cx = Context::with_observer(&mut ws, bus.as_ref());
    host.load_with_context(&cx, include_str!("autocmd/ex.lua"))
        .unwrap();
    assert!(cx.take_messages().is_empty());
    assert!(host.take_option_changes().is_empty());
}

#[test]
fn saved_buffer_accessors_follow_scopes_and_legacy_pending_text_is_not_discarded() {
    let host = PluginHost::new().unwrap();
    host.load("saved_get=vibook.buf.get_text;saved_set=vibook.buf.set_text;saved_cmd=vibook.cmd")
        .unwrap();
    let mut ws = Workspace::new(Document::new("first"));
    let cx = Context::new(&mut ws);
    host.load_with_context(
        &cx,
        "assert(saved_get() == 'first');saved_set('second');assert(saved_get() == 'second')",
    )
    .unwrap();
    host.load("assert(saved_get() == 'second');saved_set('pending')")
        .unwrap();
    assert!(host.load_with_context(&cx, "saved_set('lost')").is_err());
    assert_eq!(host.buffer_text(), "pending");
    host.set_buffer("second", 0);
    host.load_with_context(
        &cx,
        "assert(saved_get() == 'second');saved_cmd('enew');assert(saved_get() == '')",
    )
    .unwrap();
}

#[test]
fn native_ex_filtering_deleted_groups_and_replacement_shapes() {
    let bus = Rc::new(Bus::new(false));
    let host = PluginHost::new_with_bus(bus.clone()).unwrap();
    let mut ws = Workspace::new(Document::new(""));
    let cx = Context::with_observer(&mut ws, bus.as_ref());
    host.load_with_context(&cx, include_str!("autocmd/ex_shapes.lua"))
        .unwrap();
    assert!(
        cx.take_messages()
            .iter()
            .all(|message| message.contains("W19"))
    );
}

#[test]
fn failed_ex_preserves_buffer_switches_and_restores_scoped_services() {
    let host = PluginHost::new().unwrap();
    let mut doc = Document::new("seed");
    doc.name = "first.md".into();
    let mut ws = Workspace::new(doc);
    let cx = Context::new(&mut ws);
    assert!(
        host.load_with_context(&cx, "vibook.cmd('enew | Invalid')")
            .is_err()
    );
    host.load_with_context(
        &cx,
        "assert(vibook.buf.get_text() == '');vibook.buf.set_text('changed')",
    )
    .unwrap();
    host.load("assert(vibook.buf.get_text() == 'changed');assert(not pcall(vibook.cmd,'enew'))")
        .unwrap();
}

#[test]
fn an_ex_write_inside_a_lua_callback_finishes_before_the_next_text_change() {
    use std::fs;
    let root = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("ex-lua-write");
    fs::create_dir_all(&root).unwrap();
    let file = root.join("note.md");
    fs::write(&file, "initial").unwrap();
    let bus = Rc::new(Bus::new(false));
    let host = PluginHost::new_with_bus(bus.clone()).unwrap();
    let mut ws = Workspace::new(Document::new(""));
    let cx = Context::with_observer(&mut ws, bus.as_ref());
    cx.execute(&format!("edit {}", file.display())).unwrap();
    host.load_with_context(
        &cx,
        r#"
  vibook.create_autocmd('User',{callback=function()
    vibook.buf.set_text('saved')
    vibook.cmd('write')
    vibook.buf.set_text('later')
  end})
  vibook.cmd('doautocmd <nomodeline> User Save')
  assert(vibook.buf.get_text() == 'later')
 "#,
    )
    .unwrap();
    assert_eq!(fs::read_to_string(&file).unwrap(), "saved");
    assert!(cx.take_messages().is_empty());
}

#[test]
fn ex_stops_after_a_handler_that_records_nested_errors_even_if_the_lua_handler_returns_normally() {
    let bus = Rc::new(Bus::new(false));
    let host = PluginHost::new_with_bus(bus.clone()).unwrap();
    let mut ws = Workspace::new(Document::new(""));
    let cx = Context::with_observer(&mut ws, bus.as_ref());
    host.load_with_context(&cx,r#"
  local continued,second=false,false
  vibook.create_autocmd('OptionSet',{pattern='timeoutlen',callback=function()error('expected option')end})
  vibook.create_autocmd('User',{pattern='Outer',nested=true,callback=function()
    vibook.o.timeoutlen=17;continued=true
  end})
  vibook.create_autocmd('User',{pattern='Outer',callback=function()second=true end})
  assert(not pcall(vibook.cmd,'doautocmd <nomodeline> User Outer'))
  assert(continued and not second)
  continued=false
  vibook.exec_autocmds('User',{pattern='Outer',modeline=false})
  assert(continued and second)
 "#).unwrap();
    assert_eq!(cx.take_messages().len(), 2);
}

#[test]
fn nested_api_errors_abort_outer_ex_but_api_callbacks_continue() {
    let bus = Rc::new(Bus::new(false));
    let host = PluginHost::new_with_bus(bus.clone()).unwrap();
    let mut ws = Workspace::new(Document::new(""));
    let cx = Context::with_observer(&mut ws, bus.as_ref());
    host.load_with_context(
        &cx,
        r#"
      local inner_second,continued,outer_second=false,false,false
      vibook.create_autocmd('User',{pattern='Inner',callback=function()error('expected inner')end})
      vibook.create_autocmd('User',{pattern='Inner',callback=function()inner_second=true end})
      vibook.create_autocmd('User',{pattern='Outer',callback=function()
        vibook.exec_autocmds('User',{pattern='Inner',modeline=false});continued=true
      end})
      vibook.create_autocmd('User',{pattern='Outer',callback=function()outer_second=true end})
      assert(not pcall(vibook.cmd,'doautocmd <nomodeline> User Outer'))
      assert(inner_second and continued and not outer_second)
    "#,
    )
    .unwrap();
    assert_eq!(cx.take_messages().len(), 1);
}
