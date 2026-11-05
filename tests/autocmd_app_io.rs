mod common;
use std::fs;
use v_ide_editor::document::Document;
use v_ide_tui::app::App;

#[test]
fn automatic_visible_reload_emits_read_events_without_enter() {
    let root = common::scratch("autocmd-auto-read");
    let path = root.join("a.md");
    fs::write(&path, "before").unwrap();
    let mut app = App::new(Document::open(&path).unwrap());
    app.workspace.set_cwd(root);
    common::load_lua(
        &mut app,
        "autocmd-auto-read",
        r#"
   local trace={}
   vibook.create_autocmd({'BufReadPre','BufReadPost','FileType','BufEnter','BufWinEnter'},{callback=function(e)trace[#trace+1]=e.event end})
   vibook.command('Capture',function()vibook.buf.set_text(table.concat(trace,','))end)
 "#,
    );
    fs::write(&path, "after").unwrap();
    assert!(v_ide_tui::checktime::run(&mut app));
    assert_eq!(app.workspace.current().editor.text(), "after");
    assert!(!app.workspace.current().is_dirty());
    v_ide_tui::command_line::run(&mut app, "Capture");
    assert_eq!(
        app.workspace.current().editor.text(),
        "BufReadPre,BufReadPost,FileType"
    );
}

#[test]
fn automatic_hidden_reload_uses_the_target_context_and_restores_the_view() {
    let root = common::scratch("autocmd-auto-hidden");
    fs::write(root.join("a.md"), "before").unwrap();
    fs::write(root.join("b.md"), "visible").unwrap();
    let mut app = App::new(Document::new(""));
    app.workspace.set_cwd(root.clone());
    app.open_rel("a.md");
    let hidden = app.workspace.current_id();
    app.open_rel("b.md");
    let visible = app.workspace.current_id();
    app.ui.read.top = 2;
    common::load_lua(
        &mut app,
        "autocmd-auto-hidden",
        r#"
        vibook.create_autocmd({'BufLeave','BufEnter'}, {callback=function() error('service frame emitted lifecycle') end})
        vibook.create_autocmd('BufReadPost', {pattern='a.md', callback=function(e)
          assert(vibook.buf.get_text()=='on disk')
          vibook.buf.set_text('post edit')
        end})
        "#,
    );
    fs::write(root.join("a.md"), "on disk").unwrap();

    assert!(v_ide_tui::checktime::run(&mut app));

    assert_eq!(app.workspace.current_id(), visible);
    assert_eq!(app.workspace.current().editor.text(), "visible");
    assert_eq!(app.ui.read.top, 2);
    let doc = app.workspace.doc(hidden).unwrap();
    assert_eq!(doc.editor.text(), "post edit");
    assert!(!doc.is_dirty());
    assert!(
        !app.ui
            .messages
            .history()
            .any(|line| line.contains("service frame emitted lifecycle"))
    );
}

#[test]
fn automatic_read_pre_error_preserves_text_and_reports_once_per_disk_change() {
    let root = common::scratch("autocmd-auto-pre-error");
    let path = root.join("a.md");
    fs::write(&path, "original").unwrap();
    let mut app = App::new(Document::open(&path).unwrap());
    app.workspace.set_cwd(root);
    common::load_lua(
        &mut app,
        "autocmd-auto-pre-error",
        r#"
        vibook.create_autocmd('BufReadPre',{callback=function()error('auto pre marker')end})
        "#,
    );
    fs::write(&path, "new").unwrap();

    v_ide_tui::checktime::run(&mut app);
    v_ide_tui::checktime::run(&mut app);

    assert_eq!(app.workspace.current().editor.text(), "original");
    assert!(!app.workspace.current().is_dirty());
    assert_eq!(
        app.ui
            .messages
            .history()
            .filter(|line| line.contains("auto pre marker"))
            .count(),
        1
    );
}

#[cfg(unix)]
#[test]
fn mode_only_disk_change_does_not_run_read_handlers() {
    use std::os::unix::fs::PermissionsExt;

    let root = common::scratch("autocmd-auto-mode-only");
    let path = root.join("a.md");
    fs::write(&path, "original").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
    let mut app = App::new(Document::open(&path).unwrap());
    app.workspace.set_cwd(root);
    common::load_lua(
        &mut app,
        "autocmd-auto-mode-only",
        r#"
        vibook.create_autocmd({'BufReadPre','BufReadPost','FileType'},{callback=function()error('unexpected reread')end})
        "#,
    );
    fs::set_permissions(&path, fs::Permissions::from_mode(0o444)).unwrap();

    assert!(v_ide_tui::checktime::run(&mut app));

    assert!(app.workspace.current().readonly());
    assert_eq!(app.workspace.current().editor.text(), "original");
    assert!(
        !app.ui
            .messages
            .history()
            .any(|line| line.contains("unexpected reread"))
    );
}
