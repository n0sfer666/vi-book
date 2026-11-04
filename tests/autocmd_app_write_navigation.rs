mod common;

use std::fs;
use v_ide_editor::document::Document;
use v_ide_tui::{app::App, command_line};

#[test]
fn note_save_preserves_navigation_when_note_open_returns_to_the_written_buffer() {
    let root = common::scratch("write-note-open-roundtrip");
    fs::write(root.join("a.md"), "first").unwrap();
    fs::write(root.join("b.md"), "second").unwrap();
    let mut app = App::new(Document::new(""));
    app.workspace.set_cwd(root);
    app.open_rel("a.md");
    let written = app.workspace.current_id();
    common::load_lua(
        &mut app,
        "write-note-open-roundtrip",
        r#"
        vibook.create_autocmd('BufWritePost',{callback=function()
          vibook.cmd('enew')
          vibook.buf.set_text('post buffer')
        end})
        vibook.on('note_save',function()vibook.open('b.md')end)
        vibook.on('note_open',function()
          assert(vibook.buf.get_text()=='second')
          vibook.cmd('edit a.md')
        end)
        "#,
    );

    command_line::run(&mut app, "w");

    assert_eq!(app.workspace.current_id(), written);
    assert_eq!(app.workspace.current().editor.text(), "first");
    assert!(
        app.workspace
            .documents()
            .any(|(_, doc)| doc.editor.text() == "post buffer")
    );
}

#[test]
fn note_save_preserves_roundtrip_when_buf_enter_reports_an_error() {
    let root = common::scratch("write-buf-enter-roundtrip");
    fs::write(root.join("a.md"), "first").unwrap();
    fs::write(root.join("b.md"), "second").unwrap();
    let mut app = App::new(Document::new(""));
    app.workspace.set_cwd(root);
    app.open_rel("b.md");
    app.open_rel("a.md");
    let written = app.workspace.current_id();
    common::load_lua(
        &mut app,
        "write-buf-enter-roundtrip",
        r#"
        vibook.create_autocmd('BufWritePost',{callback=function()
          vibook.cmd('enew')
          vibook.buf.set_text('post buffer')
        end})
        vibook.on('note_save',function()vibook.open('b.md')end)
        vibook.create_autocmd('BufEnter',{pattern='b.md',callback=function()
          vibook.cmd('edit a.md')
          error('round trip marker')
        end})
        "#,
    );

    command_line::run(&mut app, "w");

    assert_eq!(app.workspace.current_id(), written);
    assert_eq!(app.workspace.current().editor.text(), "first");
    assert!(app.ui.message.contains("round trip marker"));
}
