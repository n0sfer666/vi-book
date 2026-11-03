mod common;

use v_ide_editor::{
    document::Document,
    options::{Access, Name, Value},
};
use v_ide_tui::{app::App, command_line, requests};

#[test]
fn configuration_and_plugin_commands_share_native_autocmds_and_live_core_state() {
    let mut app = App::new(Document::new("seed"));
    common::load_lua(
        &mut app,
        "native-live-app",
        r#"
      local seen=0
      vibook.create_autocmd('OptionSet',{pattern='timeoutlen',callback=function()
        seen=seen+1;vibook.cmd('set readonly')
      end})
      vibook.create_autocmd('User',{buffer=0,callback=function()vibook.buf.set_text('native')end})
      vibook.command('live',function()
        vibook.o.timeoutlen=17
        assert(seen==1 and vibook.bo.readonly)
        vibook.cmd('doautocmd <nomodeline> User')
        assert(vibook.buf.get_text()=='native')
        vibook.cmd('enew')
        assert(vibook.buf.get_text()=='')
        vibook.buf.set_text('new current')
      end)
    "#,
    );
    assert!(app.ui.message.is_empty(), "{}", app.ui.message);
    command_line::run(&mut app, "live");
    assert!(app.ui.message.is_empty(), "{}", app.ui.message);
    assert_eq!(app.workspace.current().editor.text(), "new current");
    assert_eq!(
        app.workspace.option(Name::Timeoutlen, Access::Effective),
        Value::Number(17)
    );
    assert_eq!(app.workspace.pool()[0].doc.editor.text(), "native");
}

#[test]
fn legacy_note_callbacks_use_live_context_and_preserve_writes_before_errors() {
    let mut app = App::new(Document::new("seed"));
    common::load_lua(
        &mut app,
        "native-live-legacy",
        r#"
      vibook.on('note_save',function()
        vibook.cmd('set readonly');assert(vibook.bo.readonly)
        vibook.buf.set_text('kept');error('expected live failure')
      end)
    "#,
    );
    requests::emit(
        &mut app,
        v_ide_editor::module::Event::NoteSave("a.md".into()),
    );
    assert_eq!(app.workspace.current().editor.text(), "kept");
    assert!(app.workspace.current().readonly());
    assert!(
        app.ui.message.contains("expected live failure"),
        "{}",
        app.ui.message
    );
}

#[test]
fn app_ex_and_native_lua_use_one_bus_and_buffer_switches_restore_ui_state() {
    let mut doc = Document::new("seed");
    doc.name = "first.md".into();
    let mut app = App::new(doc);
    common::load_lua(
        &mut app,
        "native-ex-app",
        r#"
      vibook.create_autocmd('OptionSet',{pattern='timeoutlen',callback=function()vibook.cmd('set readonly')end})
      vibook.command('verify',function()assert(vibook.bo.readonly and vibook.wo.number)end)
      vibook.command('fresh',function()vibook.cmd('enew')end)
    "#,
    );
    command_line::run(&mut app, "autocmd User Mark set number");
    command_line::run(&mut app, "doautocmd <nomodeline> User Mark");
    command_line::run(&mut app, "set timeoutlen=19");
    command_line::run(&mut app, "verify");
    assert!(!app.ui.message.contains("error"), "{}", app.ui.message);
    assert!(app.workspace.current().readonly());
    app.ui.read.top = 11;
    app.ui.left = 7;
    command_line::run(&mut app, "fresh");
    assert_eq!(app.ui.read.top, 0);
    assert_eq!(app.ui.left, 0);
    app.switch_to_buffer(0);
    assert_eq!(app.ui.read.top, 11);
    assert_eq!(app.ui.left, 7);
}

#[test]
fn autocmd_commands_resolve_plugin_commands_inline_and_native_open_intents_are_applied() {
    let root = common::scratch("native-inline-command");
    std::fs::write(root.join("b.md"), "opened").unwrap();
    let mut app = App::new(Document::new("seed"));
    app.workspace.set_cwd(root);
    common::load_lua(
        &mut app,
        "native-inline-command",
        r#"
   vibook.command('inner',function()vibook.buf.set_text('inner')end)
   vibook.create_autocmd('User',{pattern='Inline',command='inner'})
   vibook.create_autocmd('User',{pattern='Open',callback=function()vibook.open('b.md')end})
   vibook.command('outer',function()vibook.cmd('doautocmd <nomodeline> User Inline');assert(vibook.buf.get_text()=='inner')end)
 "#,
    );
    command_line::run(&mut app, "outer");
    assert_eq!(app.workspace.current().editor.text(), "inner");
    command_line::run(&mut app, "doautocmd <nomodeline> User Open");
    assert_eq!(app.workspace.current().editor.text(), "opened");
}
