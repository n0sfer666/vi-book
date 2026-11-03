mod common;
use v_ide_editor::document::Document;
use v_ide_tui::{app::App, command_line};

fn app(label: &str, script: &str) -> App {
    let mut doc = Document::new("old");
    doc.name = "old.md".into();
    let mut app = App::new(doc);
    app.workspace.set_cwd(common::scratch(label));
    common::load_lua(&mut app, label, script);
    app
}

#[test]
fn enew_and_buffer_commands_deliver_native_lifecycle_phases_inline() {
    let mut app = app(
        "buffer-native-phases",
        r#"
   local trace={}
   vibook.create_autocmd({'BufNew','BufAdd','BufLeave','BufWinLeave','BufHidden','BufDelete','BufWipeout','BufEnter','BufWinEnter'}, {callback=function(e)trace[#trace+1]=e.event end})
   vibook.command('Fresh',function()
     vibook.cmd('enew')
     assert(table.concat(trace,',')=='BufNew,BufAdd,BufLeave,BufWinLeave,BufHidden,BufEnter,BufWinEnter',table.concat(trace,','))
     trace={}
   end)
   vibook.command('Capture',function()vibook.buf.set_text(table.concat(trace,','))end)
 "#,
    );
    command_line::run(&mut app, "Fresh");
    assert!(!app.ui.message.contains("error"), "{}", app.ui.message);
    command_line::run(&mut app, "buffer 1");
    command_line::run(&mut app, "Capture");
    assert_eq!(
        app.workspace.current().editor.text(),
        "BufLeave,BufWinLeave,BufDelete,BufWipeout,BufEnter,BufWinEnter"
    );
}

#[test]
fn leave_handler_edits_keep_the_previously_empty_buffer_without_delete_events() {
    let mut app = app(
        "buffer-rescued-on-leave",
        r#"
   vibook.cmd('enew')
   local trace={}
   vibook.create_autocmd({'BufLeave','BufWinLeave','BufHidden','BufDelete','BufWipeout','BufEnter','BufWinEnter'}, {callback=function(e)trace[#trace+1]=e.event end})
   vibook.create_autocmd('BufLeave',{buffer=0,callback=function()vibook.buf.set_text('rescued')end})
   vibook.command('Capture',function()vibook.buf.set_text(table.concat(trace,','))end)
 "#,
    );
    let rescued = app.workspace.current_id();
    app.switch_to_buffer(0);
    assert_eq!(app.workspace.current().name, "old.md");
    assert_eq!(app.workspace.doc(rescued).unwrap().editor.text(), "rescued");
    assert!(app.workspace.doc(rescued).unwrap().is_dirty());
    command_line::run(&mut app, "Capture");
    assert_eq!(
        app.workspace.current().editor.text(),
        "BufLeave,BufWinLeave,BufHidden,BufEnter,BufWinEnter"
    );
}

#[test]
fn rename_events_see_old_then_new_names_and_do_not_rename_a_callback_selected_buffer() {
    let mut app = app(
        "buffer-native-rename",
        r#"
   local trace={}
   vibook.create_autocmd({'BufFilePre','BufFilePost'},{callback=function(e)trace[#trace+1]=e.event..':'..e.file:match('[^/]+$')end})
   vibook.command('Capture',function()vibook.buf.set_text(table.concat(trace,','))end)
 "#,
    );
    let original = app.workspace.current_id();
    command_line::run(&mut app, "file renamed.md");
    command_line::run(&mut app, "Capture");
    assert_eq!(
        app.workspace.current().editor.text(),
        "BufFilePre:old.md,BufFilePost:renamed.md"
    );
    common::load_lua(
        &mut app,
        "buffer-rename-reentrant",
        "vibook.create_autocmd('BufFilePre',{once=true,callback=function()vibook.cmd('enew')end})",
    );
    command_line::run(&mut app, "file wrong.md");
    assert!(app.workspace.current().name.is_empty());
    assert_eq!(app.workspace.doc(original).unwrap().name, "renamed.md");
    assert!(!app.ui.message.contains("error"), "{}", app.ui.message);
}

#[test]
fn enter_handler_navigation_redirects_win_enter_to_the_actual_current_buffer() {
    let mut app = app(
        "buffer-enter-redirect",
        r#"
   vibook.cmd('enew | file next.md | buffer 1')
   local trace={}
   vibook.create_autocmd('BufEnter',{buffer=2,once=true,callback=function(e)trace[#trace+1]=e.event..':'..e.buf;vibook.cmd('buffer 1')end})
   vibook.create_autocmd('BufWinEnter',{callback=function(e)trace[#trace+1]=e.event..':'..e.buf end})
   vibook.command('Capture',function()assert(vibook.wo.number);vibook.buf.set_text(table.concat(trace,','))end)
 "#,
    );
    command_line::run(&mut app, "buffer 2 | set number");
    assert_eq!(app.workspace.current().name, "old.md");
    command_line::run(&mut app, "Capture");
    assert_eq!(
        app.workspace.current().editor.text(),
        "BufEnter:2,BufWinEnter:1"
    );
}

#[test]
fn closing_rechecks_dirty_after_leave_and_keeps_new_handler_text() {
    let mut app = app(
        "buffer-close-rescued",
        r#"
   vibook.cmd('enew')
   vibook.create_autocmd('BufLeave',{buffer=0,callback=function()vibook.buf.set_text('rescued')end})
 "#,
    );
    let closing = app.workspace.current_id();
    app.close_buffer(0);
    assert_eq!(app.workspace.current().name, "old.md");
    assert_eq!(app.workspace.doc(closing).unwrap().editor.text(), "rescued");
    assert!(app.workspace.doc(closing).unwrap().is_dirty());
    assert!(app.ui.message.contains("unsaved"), "{}", app.ui.message);
}

#[test]
fn leave_handler_navigation_completes_the_outer_chain_without_duplicate_events() {
    for (redirect, expected) in [
        (2, "BufLeave:1"),
        (3, "BufLeave:1,BufEnter:2,BufWinEnter:2"),
    ] {
        let script = r#"
   vibook.cmd('enew | file next.md | enew | file third.md | buffer 1')
   local trace={}
   vibook.create_autocmd({'BufLeave','BufWinLeave','BufHidden','BufEnter','BufWinEnter'},{callback=function(e)trace[#trace+1]=e.event..':'..e.buf end})
   vibook.create_autocmd('BufLeave',{buffer=1,once=true,callback=function()vibook.cmd('buffer REDIRECT')end})
   vibook.command('Capture',function()assert(vibook.wo.number);vibook.buf.set_text(table.concat(trace,','))end)
 "#
        .replace("REDIRECT", &redirect.to_string());
        let mut app = app(&format!("buffer-leave-redirect-{redirect}"), &script);
        command_line::run(&mut app, "set nonumber");
        command_line::run(&mut app, "buffer 2 | set number");
        assert_eq!(app.workspace.current().name, "next.md");
        command_line::run(&mut app, "Capture");
        assert_eq!(app.workspace.current().editor.text(), expected);
        assert!(!app.ui.message.contains("error"), "{}", app.ui.message);
    }
}
