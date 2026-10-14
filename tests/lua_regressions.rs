mod common;

use std::fs;
use std::path::PathBuf;

use v_ide_lua::host::PluginHost;
use v_ide_lua::runtime::Runtime;
use v_ide_tui::app::App;
use v_ide_tui::command_line;

#[test]
fn print_and_warn_are_denied() {
    let rt = Runtime::new().unwrap();
    assert!(rt.eval_bool("return print == nil").unwrap());
    assert!(rt.eval_bool("return warn == nil").unwrap());
}

fn temp_vault(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("vi-book-luareg-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

#[test]
fn note_open_hook_does_not_leave_buffer_dirty() {
    let root = temp_vault("noteopen");
    fs::write(root.join("a.md"), "BODY").unwrap();

    let host = PluginHost::new().unwrap();
    host.load("vibook.on('note_open', function() vibook.buf.set_text('[open] ' .. vibook.buf.get_text()) end)")
        .unwrap();

    let mut app = App::new(common::named_doc("", ""));
    common::open_vault(&mut app, &root);
    app.host = Some(host);
    app.open_rel("a.md");

    let text = app.workspace.current().editor.text();
    let dirty = app.workspace.current().is_dirty();
    let _ = fs::remove_dir_all(&root);
    assert_eq!(text, "[open] BODY", "note_open transform applied");
    assert!(!dirty, "freshly opened note must not be reported dirty");
}

#[test]
fn new_note_is_visible_to_lua_vault() {
    let root = temp_vault("newsync");
    let host = PluginHost::new().unwrap();
    host.load(
        "vibook.command('count', function()\n\
           vibook.buf.set_text(tostring(#vibook.vault.notes()))\n\
         end)",
    )
    .unwrap();
    let mut app = App::new(common::named_doc("", ""));
    common::open_vault(&mut app, &root);
    app.host = Some(host);
    command_line::run(&mut app, "new fresh");
    let host = app.host.as_ref().unwrap();
    host.run_command("count").unwrap();
    let count = host.buffer_text();
    let _ = fs::remove_dir_all(&root);
    assert_eq!(count, "1", "lua vault must see the note created by :new");
}

#[test]
fn daily_note_is_visible_to_lua_vault() {
    let root = temp_vault("dailysync");
    let host = PluginHost::new().unwrap();
    host.load(
        "vibook.command('count', function()\n\
           vibook.buf.set_text(tostring(#vibook.vault.notes()))\n\
         end)",
    )
    .unwrap();
    let mut app = App::new(common::named_doc("", ""));
    common::open_vault(&mut app, &root);
    app.host = Some(host);
    command_line::run(&mut app, "today");
    let host = app.host.as_ref().unwrap();
    host.run_command("count").unwrap();
    let count = host.buffer_text();
    let _ = fs::remove_dir_all(&root);
    assert_eq!(count, "1", "lua vault must see the daily created by :today");
}
