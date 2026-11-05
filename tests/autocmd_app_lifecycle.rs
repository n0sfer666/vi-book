mod common;

use std::fs;
use v_ide_editor::document::Document;
use v_ide_tui::{app::App, command_line};

const TRACE: &str = r#"
    local trace={}
    vibook.create_autocmd({'WinLeave','WinNew','WinEnter','WinClosed','BufNew','BufAdd','BufLeave',
        'BufWinLeave','BufHidden','BufReadPre','FileType','BufReadPost','BufEnter','BufWinEnter',
        'BufUnload','BufDelete','BufWipeout'}, {callback=function(e)
      if e.event=='WinClosed' then assert(tonumber(e.match) and e.match==e.file) end
      trace[#trace+1]=e.event
    end})
    vibook.command('Capture',function()vibook.buf.set_text(table.concat(trace,','));trace={} end)
"#;

fn fixtures(label: &str) -> App {
    let root = common::scratch(label);
    fs::write(root.join("a.md"), "A").unwrap();
    fs::write(root.join("b.md"), "B").unwrap();
    let mut app = App::new(Document::new(""));
    app.workspace.set_cwd(root);
    app.open_rel("a.md");
    app
}

#[test]
fn shared_split_focus_and_close_only_emit_window_events() {
    let mut app = fixtures("shared-window-events");
    let shared = app.workspace.current_id();
    common::load_lua(&mut app, "shared-window-events", TRACE);

    app.vsplit(None);
    app.focus_next();
    app.close_active();

    assert_eq!(app.pane_count(), 1);
    assert_eq!(app.workspace.current_id(), shared);
    command_line::run(&mut app, "Capture");
    assert_eq!(
        app.workspace.current().editor.text(),
        "WinLeave,WinNew,WinEnter,WinLeave,WinEnter,WinLeave,WinClosed,WinEnter"
    );
}

#[test]
fn different_file_split_reads_after_window_enter_and_focus_and_close_report_buffers() {
    let mut app = fixtures("different-window-events");
    common::load_lua(&mut app, "different-window-events", TRACE);

    app.vsplit(Some("b.md"));
    assert_eq!(app.workspace.current().editor.text(), "B");
    app.focus_next();
    assert_eq!(app.workspace.current().editor.text(), "A");
    app.close_active();
    assert_eq!(app.workspace.current().editor.text(), "B");
    command_line::run(&mut app, "Capture");

    assert_eq!(
        app.workspace.current().editor.text(),
        concat!(
            "WinLeave,WinNew,WinEnter,BufNew,BufAdd,BufLeave,BufReadPre,FileType,BufReadPost,BufEnter,BufWinEnter,",
            "BufLeave,WinLeave,WinEnter,BufEnter,",
            "BufLeave,WinLeave,WinClosed,BufWinLeave,BufHidden,WinEnter,BufEnter",
        )
    );
}

#[test]
fn only_closes_nonactive_windows_and_hides_a_shared_document_once() {
    let mut app = fixtures("only-window-events");
    app.vsplit(Some("b.md"));
    app.vsplit(None);
    app.focus_next();
    common::load_lua(&mut app, "only-window-events", TRACE);

    app.only();
    assert_eq!(app.pane_count(), 1);
    assert_eq!(app.workspace.current().editor.text(), "A");
    command_line::run(&mut app, "Capture");

    assert_eq!(
        app.workspace.current().editor.text(),
        "WinClosed,WinClosed,BufWinLeave,BufHidden"
    );
}

#[test]
fn wiping_a_hidden_document_delivers_phases_then_removes_its_registration() {
    let mut app = fixtures("hidden-wipe-events");
    let hidden = app.workspace.current_id();
    let number = v_ide_editor::execution::Context::new(&mut app.workspace)
        .buffer_number(hidden)
        .unwrap();
    app.open_rel("b.md");
    common::load_lua(
        &mut app,
        "hidden-wipe-events",
        &format!(
            r#"
            local trace={{}}
            vibook.create_autocmd({{'BufUnload','BufDelete','BufWipeout'}}, {{buffer={number},callback=function(e)
              assert(e.buf=={number} and e.file=='a.md' and vibook.buf.get_text()=='B')
              trace[#trace+1]=e.event
            end}})
            vibook.command('Capture',function()
              assert(#vibook.get_autocmds({{event={{'BufUnload','BufDelete','BufWipeout'}}}})==0)
              vibook.buf.set_text(table.concat(trace,','))
            end)
            "#,
        ),
    );

    app.close_buffer(1);
    assert!(app.workspace.doc(hidden).is_none());
    command_line::run(&mut app, "Capture");
    assert_eq!(
        app.workspace.current().editor.text(),
        "BufUnload,BufDelete,BufWipeout"
    );
}

#[test]
fn wiping_the_last_document_transitions_to_a_new_buffer() {
    let mut app = fixtures("current-wipe-events");
    let old = app.workspace.current_id();
    common::load_lua(&mut app, "current-wipe-events", TRACE);

    app.close_buffer(0);
    assert!(app.workspace.doc(old).is_none());
    assert!(app.workspace.current().name.is_empty());
    command_line::run(&mut app, "Capture");
    assert_eq!(
        app.workspace.current().editor.text(),
        "BufNew,BufAdd,BufLeave,BufWinLeave,BufUnload,BufEnter,BufWinEnter,BufDelete,BufWipeout"
    );
}

#[test]
fn focus_completes_when_buf_leave_selects_another_document() {
    let mut app = fixtures("focus-reentrant");
    fs::write(app.workspace.absolute("c.md"), "C").unwrap();
    app.open_rel("c.md");
    let third = app.workspace.current_id();
    let third = v_ide_editor::execution::Context::new(&mut app.workspace)
        .buffer_number(third)
        .unwrap();
    app.open_rel("a.md");
    let destination = app.workspace.active_window();
    app.vsplit(Some("b.md"));
    common::load_lua(
        &mut app,
        "focus-reentrant",
        &format!(
            r#"
            vibook.create_autocmd('BufLeave',{{once=true,callback=function()vibook.cmd('buffer {third}')end}})
            vibook.create_autocmd('WinLeave',{{once=true,callback=function()assert(vibook.buf.get_text()=='C')end}})
            "#,
        ),
    );

    app.focus_next();

    assert_eq!(app.workspace.active_window(), destination);
    assert_eq!(app.workspace.current().editor.text(), "A");
    assert!(!app.ui.message.contains("error"), "{}", app.ui.message);
}

#[test]
fn closing_a_window_reports_the_buffer_it_hides_after_win_closed_navigation() {
    let mut app = fixtures("close-reentrant");
    fs::write(app.workspace.absolute("c.md"), "C").unwrap();
    app.open_rel("c.md");
    let third = app.workspace.current_id();
    let third = v_ide_editor::execution::Context::new(&mut app.workspace)
        .buffer_number(third)
        .unwrap();
    app.open_rel("a.md");
    app.vsplit(Some("b.md"));
    common::load_lua(
        &mut app,
        "close-reentrant",
        &format!(
            r#"
            local hidden
            vibook.create_autocmd('WinClosed',{{once=true,callback=function()vibook.cmd('buffer {third}')end}})
            vibook.create_autocmd('BufHidden',{{callback=function(e)hidden=e.file end}})
            vibook.command('Capture',function()vibook.buf.set_text(hidden or 'none')end)
            "#,
        ),
    );

    app.close_active();

    assert_eq!(app.pane_count(), 1);
    command_line::run(&mut app, "Capture");
    assert_eq!(app.workspace.current().editor.text(), "c.md");
}

#[test]
fn close_rechecks_dirty_after_win_leave_before_closing_the_window() {
    let mut app = fixtures("window-close-dirty");
    app.vsplit(Some("b.md"));
    common::load_lua(
        &mut app,
        "window-close-dirty",
        r#"
        vibook.create_autocmd('WinLeave',{once=true,callback=function()vibook.buf.set_text('keep me')end})
        vibook.create_autocmd('WinClosed',{callback=function()error('must remain open')end})
        "#,
    );

    app.close_active();

    assert_eq!(app.pane_count(), 2);
    assert_eq!(app.workspace.current().editor.text(), "keep me");
    assert!(app.ui.message.contains("unsaved"), "{}", app.ui.message);
}

#[test]
fn focus_decides_buf_enter_before_win_enter_and_delivers_it_for_the_actual_buffer() {
    for distinct in [false, true] {
        let label = if distinct {
            "focus-enter-distinct"
        } else {
            "focus-enter-shared"
        };
        let mut app = fixtures(label);
        app.vsplit(distinct.then_some("b.md"));
        let source = app.workspace.current_id();
        let source = v_ide_editor::execution::Context::new(&mut app.workspace)
            .buffer_number(source)
            .unwrap();
        let navigate = if distinct {
            format!("buffer {source}")
        } else {
            "enew".into()
        };
        common::load_lua(
            &mut app,
            label,
            &format!(
                r#"
                local trace={{}}
                vibook.create_autocmd('WinEnter',{{once=true,callback=function()vibook.cmd('{navigate}')end}})
                vibook.create_autocmd('BufEnter',{{callback=function(e)
                  trace[#trace+1]=vibook.buf.get_text()
                end}})
                vibook.command('Capture',function()vibook.buf.set_text(table.concat(trace,','))end)
                "#,
            ),
        );

        app.focus_next();
        command_line::run(&mut app, "Capture");

        assert_eq!(
            app.workspace.current().editor.text(),
            if distinct { "B" } else { "" }
        );
    }
}

#[test]
fn close_reports_late_errors_but_still_removes_a_clean_window() {
    for phase in ["WinClosed", "BufWinLeave", "BufHidden"] {
        let label = format!("close-error-{phase}");
        let mut app = fixtures(&label);
        app.vsplit(Some("b.md"));
        common::load_lua(
            &mut app,
            &label,
            &format!(
                r#"
                vibook.create_autocmd('{phase}',{{callback=function()error('close marker')end}})
                vibook.create_autocmd({{'WinEnter','BufEnter'}},{{callback=function()error('unexpected enter')end}})
                "#,
            ),
        );

        app.close_active();

        assert_eq!(app.pane_count(), 1);
        assert_eq!(app.workspace.current().editor.text(), "A");
        assert!(
            app.ui.message.contains("close marker"),
            "{}",
            app.ui.message
        );
        assert!(
            !app.ui
                .messages
                .history()
                .any(|line| line.contains("unexpected enter"))
        );
    }
}

#[test]
fn wipe_tracks_the_unloaded_buffer_when_buf_new_navigates() {
    let mut app = fixtures("wipe-bufnew-navigation");
    let closing = app.workspace.current_id();
    common::load_lua(
        &mut app,
        "wipe-bufnew-navigation",
        r#"
        local unloaded={}
        vibook.create_autocmd('BufNew',{once=true,callback=function()vibook.cmd('enew | file callback.md')end})
        vibook.create_autocmd('BufUnload',{callback=function(e)unloaded[#unloaded+1]=e.file end})
        vibook.command('Capture',function()vibook.buf.set_text(table.concat(unloaded,','))end)
        "#,
    );

    app.close_buffer(0);

    assert!(app.workspace.doc(closing).is_none());
    assert!(app.workspace.current().name.is_empty());
    assert!(
        app.workspace
            .documents()
            .any(|(_, doc)| doc.name == "callback.md")
    );
    command_line::run(&mut app, "Capture");
    assert_eq!(app.workspace.current().editor.text(), "callback.md,a.md");
}

#[test]
fn only_closes_remaining_clean_windows_without_events_after_a_late_error() {
    for phase in ["WinClosed", "BufWinLeave", "BufHidden"] {
        let label = format!("only-error-{phase}");
        let mut app = fixtures(&label);
        fs::write(app.workspace.absolute("c.md"), "C").unwrap();
        app.vsplit(Some("b.md"));
        app.vsplit(Some("c.md"));
        common::load_lua(
            &mut app,
            &label,
            &format!(
                r#"
                local closed=0
                vibook.create_autocmd('WinClosed',{{callback=function()closed=closed+1 end}})
                vibook.create_autocmd('{phase}',{{callback=function()error('only marker')end}})
                vibook.command('Capture',function()vibook.buf.set_text(tostring(closed))end)
                "#,
            ),
        );

        app.only();

        assert_eq!(app.pane_count(), 1);
        assert_eq!(app.workspace.current().editor.text(), "C");
        assert!(app.ui.message.contains("only marker"), "{}", app.ui.message);
        command_line::run(&mut app, "Capture");
        assert_eq!(app.workspace.current().editor.text(), "1");
    }
}
