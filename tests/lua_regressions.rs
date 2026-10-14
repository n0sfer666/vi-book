mod common;

use std::fs;
use std::path::PathBuf;

use v_ide_lua::runtime::Runtime;
use v_ide_tui::app::App;
use v_ide_tui::command_line;

const COUNT: &str = "vibook.command('count', function()\n\
                       vibook.buf.set_text(tostring(#vibook.vault.notes()))\n\
                     end)";

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

    let mut app = App::new(common::named_doc("", ""));
    common::open_vault(&mut app, &root);
    common::load_lua(
        &mut app,
        "reg-noteopen",
        "vibook.on('note_open', function() vibook.buf.set_text('[open] ' .. vibook.buf.get_text()) end)",
    );
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
    let mut app = App::new(common::named_doc("", ""));
    common::open_vault(&mut app, &root);
    common::load_lua(&mut app, "reg-newsync", COUNT);
    command_line::run(&mut app, "new fresh");
    command_line::run(&mut app, "count");
    let count = app.workspace.current().editor.text();
    let _ = fs::remove_dir_all(&root);
    assert_eq!(count, "1", "lua vault must see the note created by :new");
}

#[test]
fn daily_note_is_visible_to_lua_vault() {
    let root = temp_vault("dailysync");
    let mut app = App::new(common::named_doc("", ""));
    common::open_vault(&mut app, &root);
    common::load_lua(&mut app, "reg-dailysync", COUNT);
    command_line::run(&mut app, "today");
    command_line::run(&mut app, "count");
    let count = app.workspace.current().editor.text();
    let _ = fs::remove_dir_all(&root);
    assert_eq!(count, "1", "lua vault must see the daily created by :today");
}

#[test]
fn note_open_hook_opening_another_note_keeps_first_clean() {
    let root = temp_vault("hookopen");
    fs::write(root.join("a.md"), "A").unwrap();
    fs::write(root.join("b.md"), "B").unwrap();
    let mut app = App::new(common::named_doc("", ""));
    common::open_vault(&mut app, &root);
    common::load_lua(
        &mut app,
        "reg-hookopen",
        "vibook.on('note_open', function()\n\
           if vibook.buf.get_text() == 'A' then\n\
             vibook.buf.set_text('A!')\n\
             vibook.open('b.md')\n\
           end\n\
         end)",
    );
    app.open_rel("a.md");
    let current = app.workspace.current().name.clone();
    let first = app
        .workspace
        .pool()
        .iter()
        .find(|b| b.doc.name == "a.md")
        .map(|b| (b.doc.editor.text(), b.doc.is_dirty()));
    let _ = fs::remove_dir_all(&root);
    assert_eq!(current, "b.md");
    assert_eq!(first, Some((String::from("A!"), false)));
}

#[test]
fn note_save_hook_runs_after_write() {
    let root = temp_vault("notesave");
    fs::write(root.join("a.md"), "A").unwrap();
    let mut app = App::new(common::named_doc("", ""));
    common::open_vault(&mut app, &root);
    common::load_lua(
        &mut app,
        "reg-notesave",
        "vibook.on('note_save', function() vibook.buf.set_text('saved') end)",
    );
    app.open_rel("a.md");
    command_line::run(&mut app, "w");
    let text = app.workspace.current().editor.text();
    let disk = fs::read_to_string(root.join("a.md")).unwrap();
    let _ = fs::remove_dir_all(&root);
    assert_eq!(text, "saved");
    assert_eq!(disk, "A");
}
