mod common;
use std::{fs, path::PathBuf};
use v_ide_editor::document::Document;
use v_ide_tui::{app::App, command_line};

fn setup(label: &str, script: &str) -> (App, PathBuf) {
    let root = common::scratch(label);
    let path = root.join("a.md");
    fs::write(&path, "disk\n# vim: set ro:").unwrap();
    let mut doc = Document::new("old");
    doc.name = "old.md".into();
    let mut app = App::new(doc);
    app.workspace.set_cwd(root);
    common::load_lua(&mut app, label, script);
    (app, path)
}

#[test]
fn opening_file_runs_read_and_filetype_phases_before_enter_and_modeline() {
    let (mut app, _) = setup(
        "read-open-phases",
        r#"
   vibook.o.modeline=true
   local trace={}
   local function add(e)trace[#trace+1]=e.event end
   vibook.create_autocmd({'BufNew','BufAdd','BufLeave','BufReadPre','FileType','BufReadPost','BufEnter','BufWinEnter','OptionSet'}, {callback=function(e)
     if e.event=='FileType' then assert(e.match=='markdown' and e.file=='a.md') end
     if e.event=='BufReadPre' then assert(vibook.buf.get_text()=='') end
     if e.event=='BufReadPost' then assert(vibook.buf.get_text():find('disk',1,true)) end
     if e.event=='BufEnter' then assert(not vibook.o.readonly) end
     if e.event=='OptionSet' and e.match=='readonly' then assert(vibook.o.readonly) end
     add(e)
   end})
   vibook.create_autocmd('BufReadPre',{callback=function()vibook.buf.set_text('pre')end})
   vibook.create_autocmd('BufReadPost',{callback=function()vibook.buf.set_text(vibook.buf.get_text()..'\npost')end})
   vibook.create_autocmd('BufEnter',{callback=function()vibook.buf.set_text(vibook.buf.get_text()..'\nenter')end})
   vibook.command('Capture',function()vibook.buf.set_text(table.concat(trace,','))end)
 "#,
    );
    app.open_rel("a.md");
    assert_eq!(
        app.workspace.current().editor.text(),
        "disk\n# vim: set ro:\npre\npost\nenter"
    );
    assert!(app.workspace.current().is_dirty());
    assert!(app.workspace.current().readonly());
    command_line::run(&mut app, "Capture");
    assert_eq!(
        app.workspace.current().editor.text(),
        "BufNew,BufAdd,BufLeave,BufReadPre,FileType,BufReadPost,BufEnter,OptionSet,BufWinEnter"
    );
}

#[test]
fn explicit_reload_runs_read_phases_and_preserves_existing_failure_contract() {
    let (mut app, path) = setup(
        "read-reload-phases",
        r#"
   vibook.o.modeline=true
   local trace={}
   vibook.create_autocmd({'BufReadPre','FileType','BufReadPost','BufEnter','BufWinEnter'},{callback=function(e)trace[#trace+1]=e.event end})
   vibook.create_autocmd('BufReadPost',{callback=function()vibook.buf.set_text(vibook.buf.get_text()..'\npost')end})
   vibook.command('Capture',function()vibook.buf.set_text(table.concat(trace,','))end)
   vibook.command('Clear',function()trace={} end)
 "#,
    );
    app.open_rel("a.md");
    command_line::run(&mut app, "Clear");
    let id = app.workspace.current_id();
    app.workspace.current_mut().editor.replace_all("pending");
    fs::write(&path, "fresh").unwrap();
    command_line::run(&mut app, "e!");
    assert_eq!(app.workspace.current_id(), id);
    assert_eq!(app.workspace.current().editor.text(), "fresh\npost");
    assert!(!app.workspace.current().is_dirty());
    command_line::run(&mut app, "Capture");
    assert_eq!(
        app.workspace.current().editor.text(),
        "BufReadPre,FileType,BufReadPost,BufEnter,BufWinEnter"
    );
}

#[test]
fn pre_error_keeps_new_named_buffer_empty_and_skips_post_and_enter() {
    let (mut app, _) = setup(
        "read-pre-error",
        r#"
   vibook.create_autocmd({'BufReadPre','FileType','BufReadPost','BufEnter','BufWinEnter'},{callback=function(e)
     if e.event=='BufReadPre' then error('read pre marker') end
   end})
 "#,
    );
    app.open_rel("a.md");
    assert_eq!(app.workspace.current().name, "a.md");
    assert_eq!(app.workspace.current().editor.text(), "");
    assert!(
        app.ui.message.contains("read pre marker"),
        "{}",
        app.ui.message
    );
}

#[test]
fn post_error_keeps_read_content_saved_and_skips_enter() {
    let (mut app, _) = setup(
        "read-post-error",
        r#"
   vibook.create_autocmd('BufReadPost',{callback=function()error('read post marker')end})
 "#,
    );
    app.open_rel("a.md");
    assert!(app.workspace.current().editor.text().starts_with("disk"));
    assert!(!app.workspace.current().is_dirty());
    assert!(
        app.ui.message.contains("read post marker"),
        "{}",
        app.ui.message
    );
}

#[test]
fn reload_pre_uses_a_clean_empty_buffer_and_keeps_handler_readonly() {
    let (mut app, _) = setup("read-pre-nested-base", "");
    app.open_rel("a.md");
    common::load_lua(
        &mut app,
        "read-pre-nested-edit",
        r#"
        vibook.create_autocmd('BufReadPre',{once=true,nested=true,callback=function()
          assert(vibook.buf.get_text()=='')
          vibook.cmd('edit')
          assert(vibook.buf.get_text():find('disk',1,true))
          vibook.bo.readonly=true
        end})
        "#,
    );
    command_line::run(&mut app, "e!");
    assert!(!app.ui.message.contains("E37"), "{}", app.ui.message);
    assert!(app.workspace.current().readonly());
}

#[test]
fn filetype_navigation_still_runs_post_in_the_read_buffer() {
    let root = common::scratch("read-filetype-nav");
    fs::write(root.join("first.txt"), "previous").unwrap();
    fs::write(root.join("a.md"), "loaded").unwrap();
    let mut app = App::new(Document::new(""));
    app.workspace.set_cwd(root);
    app.open_rel("first.txt");
    let previous = app.workspace.current_id();
    let number = v_ide_editor::execution::Context::new(&mut app.workspace)
        .buffer_number(previous)
        .unwrap();
    common::load_lua(
        &mut app,
        "read-filetype-nav",
        &format!(
            r#"
            vibook.create_autocmd('FileType',{{pattern='markdown',once=true,nested=true,callback=function()
              vibook.cmd('buffer {}')
            end}})
            vibook.create_autocmd('BufReadPost',{{pattern='a.md',callback=function()
              assert(vibook.buf.get_text()=='loaded')
              vibook.buf.set_text('post on target')
            end}})
            "#,
            number
        ),
    );

    app.open_rel("a.md");

    assert_eq!(app.workspace.current_id(), previous);
    assert_eq!(app.workspace.current().editor.text(), "previous");
    let target = app
        .workspace
        .find_path(&app.workspace.absolute("a.md"))
        .unwrap();
    let doc = app.workspace.doc(target).unwrap();
    assert_eq!(doc.editor.text(), "post on target");
    assert!(!doc.is_dirty());
}
