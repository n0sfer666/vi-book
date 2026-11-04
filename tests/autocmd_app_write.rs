mod common;
use std::{fs, path::PathBuf};
use v_ide_editor::document::Document;
use v_ide_tui::{app::App, command_line};

fn setup(label: &str, script: &str) -> (App, PathBuf) {
    let root = common::scratch(label);
    let path = root.join("a.md");
    fs::write(&path, "disk").unwrap();
    let mut app = App::new(Document::new(""));
    app.workspace.set_cwd(root);
    app.open_rel("a.md");
    common::load_lua(&mut app, label, script);
    (app, path)
}

#[test]
fn root_and_inline_writes_share_native_phases_and_notify_legacy_hooks_once() {
    for command in ["w", "Inline"] {
        let (mut app, path) = setup(
            &format!("native-write-{command}"),
            r#"
   local trace={};local saves=0
   vibook.create_autocmd('BufWritePre',{callback=function(e)assert(e.file=='a.md');trace[#trace+1]=e.event;vibook.buf.set_text('saved')end})
   vibook.create_autocmd('BufWritePost',{callback=function(e)assert(e.file=='a.md');trace[#trace+1]=e.event;vibook.buf.set_text('post')end})
   vibook.on('note_save',function()assert(vibook.buf.get_text()=='post');saves=saves+1;trace[#trace+1]='legacy' end)
   vibook.command('Inline',function()vibook.cmd('write');assert(vibook.buf.get_text()=='post')end)
   vibook.command('Capture',function()assert(saves==1);vibook.buf.set_text(table.concat(trace,','))end)
 "#,
        );
        command_line::run(&mut app, command);
        assert_eq!(fs::read_to_string(path).unwrap(), "saved");
        assert_eq!(app.workspace.current().editor.text(), "post");
        assert!(app.workspace.current().is_dirty());
        command_line::run(&mut app, "Capture");
        assert_eq!(
            app.workspace.current().editor.text(),
            "BufWritePre,BufWritePost,legacy"
        );
    }
}

#[test]
fn pre_errors_cancel_write_post_legacy_notification_and_wq_exit() {
    let (mut app, path) = setup(
        "native-write-pre-error",
        r#"
   vibook.create_autocmd('BufWritePre',{callback=function()error('write pre marker')end})
   vibook.create_autocmd('BufWritePost',{callback=function()vibook.buf.set_text('wrong post')end})
   vibook.on('note_save',function()vibook.buf.set_text('wrong legacy')end)
 "#,
    );
    app.workspace.current_mut().editor.replace_all("pending");
    command_line::run(&mut app, "wq");
    assert!(!app.quit);
    assert_eq!(fs::read_to_string(path).unwrap(), "disk");
    assert_eq!(app.workspace.current().editor.text(), "pending");
    assert!(
        app.ui.message.contains("write pre marker"),
        "{}",
        app.ui.message
    );
}

#[test]
fn post_errors_keep_completed_write_and_legacy_notification_without_duplicate_diagnostics() {
    let (mut app, path) = setup(
        "native-write-post-error",
        r#"
   local saves=0
   vibook.create_autocmd('BufWritePost',{callback=function()error('write post marker')end})
   vibook.on('note_save',function()saves=saves+1 end)
   vibook.command('Capture',function()vibook.buf.set_text(tostring(saves))end)
 "#,
    );
    app.workspace.current_mut().editor.replace_all("written");
    command_line::run(&mut app, "w");
    assert_eq!(fs::read_to_string(path).unwrap(), "written");
    assert!(!app.workspace.current().is_dirty());
    assert!(
        app.ui.message.contains("write post marker"),
        "{}",
        app.ui.message
    );
    let history = app.ui.messages.history().collect::<Vec<_>>().join("\n");
    assert_eq!(history.matches("write post marker").count(), 1, "{history}");
    command_line::run(&mut app, "Capture");
    assert_eq!(app.workspace.current().editor.text(), "1");
}

#[test]
fn pre_handler_navigation_returns_to_the_original_buffer_and_preserves_handler_edits() {
    let (mut app, path) = setup(
        "native-write-changed-buffer",
        "vibook.create_autocmd('BufWritePre',{callback=function()vibook.cmd('enew');vibook.buf.set_text('handler text')end})",
    );
    let original = app.workspace.current_id();
    app.workspace.current_mut().editor.replace_all("pending");
    command_line::run(&mut app, "w");
    assert_eq!(fs::read_to_string(path).unwrap(), "pending");
    assert_eq!(app.workspace.current_id(), original);
    assert_eq!(app.workspace.current().name, "a.md");
    let other = app.workspace.pool().first().unwrap().id;
    assert_eq!(
        app.workspace.doc(other).unwrap().editor.text(),
        "handler text"
    );
    assert!(app.workspace.doc(other).unwrap().is_dirty());
}

#[test]
fn first_write_assigns_name_before_only_the_write_events() {
    let root = common::scratch("write-new-name-events");
    let mut app = App::new(Document::new("original"));
    app.workspace.set_cwd(root.clone());
    common::load_lua(
        &mut app,
        "write-new-name-events",
        r#"
   local trace={}
   vibook.create_autocmd({'BufFilePre','BufFilePost','BufWritePre','BufWritePost'},{callback=function(e)
     trace[#trace+1]=e.event
     if e.event=='BufWritePre' then
       assert(e.file=='new.md')
       assert(vibook.buf.get_text()=='original')
       vibook.buf.set_text('from pre')
     end
   end})
   vibook.command('Capture',function()vibook.buf.set_text(table.concat(trace,','))end)
 "#,
    );
    let original = app.workspace.current_id();
    command_line::run(&mut app, "w new.md");
    assert_eq!(app.workspace.current_id(), original);
    assert_eq!(app.workspace.current().name, "new.md");
    assert_eq!(fs::read_to_string(root.join("new.md")).unwrap(), "from pre");
    command_line::run(&mut app, "Capture");
    assert_eq!(
        app.workspace.current().editor.text(),
        "BufWritePre,BufWritePost"
    );
}
