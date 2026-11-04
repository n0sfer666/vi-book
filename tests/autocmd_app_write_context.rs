mod common;

use std::fs;
use v_ide_editor::document::Document;
use v_ide_tui::{app::App, command_line, lua_bridge};

#[test]
fn config_note_save_runs_after_config_requests_are_discarded_and_hook_requests_apply() {
    let root = common::scratch("config-note-save-order");
    fs::write(root.join("a.md"), "disk").unwrap();
    fs::write(root.join("hook.md"), "hook file").unwrap();
    fs::write(root.join("discard.md"), "discard file").unwrap();
    let mut app = App::new(Document::new(""));
    app.workspace.set_cwd(root.clone());
    app.open_rel("a.md");
    let script = r#"
      vibook.opt.leader = ','
      vibook.vault.create('config scratch')
      vibook.open('discard.md')
      vibook.on('note_save', function()
        vibook.buf.set_text('config callback')
        vibook.vault.create('hook scratch')
        vibook.open('hook.md')
      end)
      vibook.cmd('write')
    "#;
    let config = common::config_file("config-note-save-order", script);

    lua_bridge::load_config_from(&mut app, Some((config, None)));

    assert_eq!(app.leader, v_ide_keys::parse_keys(","));
    assert!(
        app.workspace.current().name.ends_with("hook.md"),
        "name={}, message={}, text={}",
        app.workspace.current().name,
        app.ui.message,
        app.workspace.current().editor.text()
    );
    assert_eq!(app.workspace.current().editor.text(), "hook file");
    assert!(
        !app.workspace
            .documents()
            .any(|(_, doc)| doc.name == "discard.md")
    );
}

#[test]
fn note_save_uses_saved_buffer_then_restores_post_navigation_view_and_pool() {
    let root = common::scratch("note-save-owned-frame");
    fs::write(root.join("a.md"), "disk").unwrap();
    fs::write(root.join("b.md"), "second").unwrap();
    let mut app = App::new(Document::new(""));
    app.workspace.set_cwd(root);
    app.open_rel("a.md");
    common::load_lua(
        &mut app,
        "note-save-owned-frame",
        r#"
      vibook.create_autocmd('BufWritePost',{callback=function()vibook.cmd('enew');vibook.buf.set_text('post buffer')end})
      vibook.on('note_save',function()
        assert(vibook.buf.get_text()=='written')
        vibook.buf.set_text('legacy edit')
      end)
    "#,
    );
    app.workspace.current_mut().editor.replace_all("written");
    app.workspace
        .current_mut()
        .editor
        .set_cursor(v_ide_text::Position::new(0, 2));
    app.ui.read.top = 13;
    app.ui.left = 5;
    let saved = app.workspace.current_id();
    app.open_rel("b.md");
    let second = app.workspace.current_id();
    app.workspace
        .current_mut()
        .editor
        .set_cursor(v_ide_text::Position::new(0, 4));
    app.ui.read.top = 7;
    app.ui.left = 3;
    app.switch_to_buffer(0);
    let window = app.workspace.active_window();
    assert_eq!(app.workspace.current_id(), saved);
    command_line::run(&mut app, "w");
    let selected = app.workspace.current_id();

    assert_ne!(selected, saved);
    assert_eq!(
        app.workspace.doc(saved).unwrap().editor.text(),
        "legacy edit"
    );
    assert_eq!(app.workspace.doc(saved).unwrap().editor.cursor().col, 2);
    assert_eq!(app.workspace.current().editor.text(), "post buffer");
    assert_eq!(app.workspace.current_id(), selected);
    assert_eq!(
        app.workspace
            .pool()
            .iter()
            .map(|buffer| buffer.id)
            .collect::<Vec<_>>(),
        vec![saved, second]
    );
    assert_eq!(app.ui.read.top, 0);
    assert_eq!(app.ui.left, 0);
    let saved_view = app.read_states.get(&(window, saved)).unwrap();
    assert_eq!((saved_view.top, saved_view.left), (13, 5));
    let second_view = app.read_states.get(&(window, second)).unwrap();
    assert_eq!((second_view.top, second_view.left), (7, 3));
}

#[test]
fn note_save_service_frame_preserves_a_clean_empty_buffer() {
    let root = common::scratch("note-save-empty-frame");
    fs::write(root.join("a.md"), "disk").unwrap();
    let mut app = App::new(Document::new(""));
    app.workspace.set_cwd(root);
    app.open_rel("a.md");
    common::load_lua(
        &mut app,
        "note-save-empty-frame",
        r#"
      vibook.create_autocmd('BufWritePost',{callback=function()vibook.cmd('enew')end})
      vibook.on('note_save',function()assert(vibook.buf.get_text()=='written')end)
    "#,
    );
    app.workspace.current_mut().editor.replace_all("written");

    command_line::run(&mut app, "w");

    assert!(app.workspace.current().name.is_empty());
    assert_eq!(app.workspace.current().editor.text(), "");
    assert!(
        app.workspace
            .documents()
            .any(|(_, doc)| doc.name.is_empty() && doc.editor.text().is_empty())
    );
}

#[test]
fn note_save_navigation_survives_hook_error() {
    let root = common::scratch("note-save-navigation-error");
    fs::write(root.join("a.md"), "disk").unwrap();
    let mut app = App::new(Document::new(""));
    app.workspace.set_cwd(root);
    app.open_rel("a.md");
    common::load_lua(
        &mut app,
        "note-save-navigation-error",
        r#"
      vibook.on('note_save',function()
        assert(vibook.buf.get_text()=='written')
        vibook.cmd('enew')
        vibook.buf.set_text('explicit navigation')
        error('note save marker')
      end)
      vibook.create_autocmd('BufWritePost',{callback=function()vibook.cmd('enew');vibook.buf.set_text('post buffer')end})
    "#,
    );
    app.workspace.current_mut().editor.replace_all("written");

    command_line::run(&mut app, "w");

    assert_eq!(app.workspace.current().editor.text(), "explicit navigation");
    assert!(
        app.ui.message.contains("note save marker"),
        "{}",
        app.ui.message
    );
    assert!(
        app.workspace
            .documents()
            .any(|(_, doc)| doc.editor.text() == "written")
    );
}

#[test]
fn failed_opens_from_note_save_restore_the_buffer_and_view() {
    for api in ["open", "open_external"] {
        let root = common::scratch(api);
        fs::write(root.join("a.md"), "disk").unwrap();
        let mut app = App::new(Document::new(""));
        app.workspace.set_cwd(root);
        app.open_rel("a.md");
        common::load_lua(
            &mut app,
            api,
            &format!("vibook.create_autocmd('BufWritePost',{{callback=function()vibook.cmd('enew');vibook.buf.set_text('post buffer')end}})
            vibook.on('note_save',function()vibook.{api}('missing.md')end)"),
        );
        app.ui.read.top = 13;
        app.ui.left = 5;
        let written = app.workspace.current_id();
        command_line::run(&mut app, "w");
        assert_ne!(app.workspace.current_id(), written, "{api}");
        assert_eq!(app.workspace.current().editor.text(), "post buffer");
        assert_eq!((app.ui.read.top, app.ui.left), (0, 0));
    }
}
