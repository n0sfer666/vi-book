mod common;
use std::fs;
use v_ide_editor::document::Document;
use v_ide_tui::{app::App, command_line};

#[test]
fn startup_emits_enter_once_and_suppresses_option_events_through_vim_enter() {
    let root = common::scratch("startup-native-boundary");
    let path = root.join("a.md");
    fs::write(&path, "disk").unwrap();
    let mut app = App::new(Document::open(&path).unwrap());
    app.workspace.set_cwd(root);
    app.begin_startup();
    common::load_lua(
        &mut app,
        "startup-native-boundary",
        &format!(
            r#"
   local trace={{}}
   local expected={:?}
   vibook.create_autocmd('OptionSet',{{pattern='timeoutlen',callback=function()trace[#trace+1]='OptionSet:'..vibook.o.timeoutlen end}})
   vibook.create_autocmd('BufEnter',{{callback=function(e)assert(e.buf==1 and e.file==expected and e.match==expected);trace[#trace+1]=e.event;vibook.o.timeoutlen=32 end}})
   vibook.create_autocmd('VimEnter',{{callback=function(e)assert(e.buf==1 and e.file==expected);trace[#trace+1]=e.event;vibook.o.timeoutlen=33 end}})
   vibook.o.timeoutlen=31
   vibook.command('Capture',function()vibook.buf.set_text(table.concat(trace,','))end)
 "#,
            path.to_string_lossy()
        ),
    );
    app.finish_startup();
    app.finish_startup();
    app.begin_startup();
    app.finish_startup();
    command_line::run(&mut app, "set timeoutlen=34");
    command_line::run(&mut app, "Capture");
    assert_eq!(
        app.workspace.current().editor.text(),
        "BufEnter,VimEnter,OptionSet:34"
    );
}

#[test]
fn startup_errors_are_reported_once_and_do_not_keep_the_app_in_startup() {
    for event in ["BufEnter", "VimEnter"] {
        let mut app = App::new(Document::new("seed"));
        app.begin_startup();
        common::load_lua(
            &mut app,
            &format!("startup-error-{event}"),
            &format!(
                r#"
   local trace={{}}
   vibook.create_autocmd({{'BufEnter','VimEnter','OptionSet'}},{{callback=function(e)trace[#trace+1]=e.event end}})
   vibook.create_autocmd('{event}',{{callback=function()error('startup failure marker')end}})
   vibook.command('Capture',function()vibook.buf.set_text(table.concat(trace,','))end)
 "#
            ),
        );
        app.finish_startup();
        command_line::run(&mut app, "set timeoutlen=17");
        let history = app.ui.messages.history().collect::<Vec<_>>().join("\n");
        assert_eq!(
            history.matches("startup failure marker").count(),
            1,
            "{history}"
        );
        command_line::run(&mut app, "Capture");
        assert_eq!(
            app.workspace.current().editor.text(),
            "BufEnter,VimEnter,OptionSet",
            "{event}"
        );
    }
}

#[test]
fn synthetic_app_configuration_retains_live_option_events_without_begin_startup() {
    let mut app = App::new(Document::new("seed"));
    common::load_lua(
        &mut app,
        "startup-synthetic-compat",
        r#"
   local trace={}
   vibook.create_autocmd({'OptionSet','VimEnter'},{callback=function(e)trace[#trace+1]=e.event end})
   vibook.o.timeoutlen=29
   vibook.command('Capture',function()vibook.buf.set_text(table.concat(trace,','))end)
 "#,
    );
    app.finish_startup();
    command_line::run(&mut app, "Capture");
    assert_eq!(app.workspace.current().editor.text(), "OptionSet,VimEnter");
}

#[test]
fn first_cli_read_finishes_modeline_and_window_entry_before_startup_enter() {
    let root = common::scratch("startup-first-read-phases");
    let path = root.join("a.md");
    fs::write(&path, "disk\n# vim: set ro:").unwrap();
    let mut app = App::new(Document::new(""));
    app.workspace.set_cwd(root);
    app.begin_startup();
    common::load_lua(
        &mut app,
        "startup-first-read-phases",
        r#"
   vibook.o.modeline=true
   local trace={}
   vibook.create_autocmd({'BufReadPre','FileType','BufReadPost','BufWinEnter','BufEnter','VimEnter'},{callback=function(e)
     if e.event=='BufReadPre' then assert(not vibook.o.readonly) end
     if e.event=='BufReadPost' then assert(not vibook.o.readonly) end
     if e.event=='BufWinEnter' or e.event=='BufEnter' or e.event=='VimEnter' then assert(vibook.o.readonly) end
     trace[#trace+1]=e.event
   end})
   vibook.command('Capture',function()vibook.buf.set_text(table.concat(trace,','))end)
 "#,
    );
    app.open_initial_rel("a.md");
    assert!(app.workspace.current().readonly());
    app.finish_startup();
    command_line::run(&mut app, "Capture");
    assert_eq!(
        app.workspace.current().editor.text(),
        "BufReadPre,FileType,BufReadPost,BufWinEnter,BufEnter,VimEnter"
    );
}
