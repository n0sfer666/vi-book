use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use v_ide_editor::document::Document;
use v_ide_editor::module::{Event, Modules, Request};
use v_ide_editor::workspace::Workspace;
use v_ide_keys::Key;
use v_ide_lua::module::LuaModule;

fn ws(text: &str) -> Workspace {
    let mut ws = Workspace::new(Document::new(text));
    ws.set_cwd(PathBuf::from("/vault"));
    ws
}

fn started(label: &str, script: &str) -> Modules {
    let mut modules = Modules::new(vec![LuaModule::registration()]);
    assert_eq!(modules.activate(&ws(""), "lua"), Vec::new());
    let path = std::env::temp_dir().join(format!(
        "vi-book-lua-module-{}-{label}.lua",
        std::process::id()
    ));
    fs::write(&path, script).unwrap();
    let lua = modules.get::<LuaModule>().expect("lua module active");
    let messages = lua.load_config(Some((path.clone(), None)));
    let _ = fs::remove_file(&path);
    assert_eq!(messages, Vec::<String>::new());
    modules
}

fn text(value: &str) -> Request {
    Request::SetText(String::from(value))
}

fn follow(modules: &mut Modules, target: &str) -> Vec<Request> {
    modules.event(&ws(""), &Event::FollowLink(String::from(target)))
}

#[test]
fn bundled_links_plugin_classifies_targets() {
    let mut modules = started("s1", "");
    assert_eq!(
        follow(&mut modules, "x.rs"),
        vec![Request::OpenExternal(String::from("x.rs"))]
    );
    assert_eq!(
        follow(&mut modules, "b.md"),
        vec![Request::Open(PathBuf::from("/vault/b.md"))]
    );
    assert_eq!(follow(&mut modules, "bare"), Vec::new());
}

#[test]
fn load_config_reports_warning_then_error() {
    let mut modules = Modules::new(vec![LuaModule::registration()]);
    modules.activate(&ws(""), "lua");
    let path =
        std::env::temp_dir().join(format!("vi-book-lua-module-{}-bad.lua", std::process::id()));
    fs::write(&path, "this is not lua").unwrap();
    let lua = modules.get::<LuaModule>().unwrap();
    let messages = lua.load_config(Some((path.clone(), Some("config: old"))));
    let _ = fs::remove_file(&path);
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0], "config: old");
    assert!(messages[1].starts_with("config error: "), "{messages:?}");
    assert_eq!(lua.load_config(None), Vec::<String>::new());
}

#[test]
fn plugin_command_is_routed_and_rewrites_buffer() {
    let mut modules = started(
        "s2",
        "vibook.command('shout', function() vibook.buf.set_text(vibook.buf.get_text():upper()) end)
         vibook.command('noop', function() end)",
    );
    let ws = ws("hi");
    assert_eq!(modules.command(&ws, "shout", ""), Some(vec![text("HI")]));
    assert_eq!(modules.command(&ws, "noop", ""), Some(Vec::new()));
    assert_eq!(modules.command(&ws, "missing", ""), None);
}

#[test]
fn plugin_sees_cursor_index() {
    let mut modules = started(
        "s3",
        "vibook.command('pos', function() vibook.buf.set_text(tostring(vibook.buf.get_cursor())) end)",
    );
    let mut ws = ws("hello");
    ws.current_mut().editor.handle_key(Key::Char('$'));
    assert_eq!(modules.command(&ws, "pos", ""), Some(vec![text("4")]));
}

#[test]
fn command_error_is_reported() {
    let mut modules = started("s4", "vibook.command('boom', function() error('bad') end)");
    let requests = modules.command(&ws("x"), "boom", "").unwrap();
    assert!(
        matches!(requests.as_slice(), [Request::Message(m)] if m.starts_with("plugin error: ")),
        "{requests:?}"
    );
}

#[test]
fn note_hooks_transform_buffer() {
    let mut modules = started("s5", 
        "vibook.on('note_open', function() vibook.buf.set_text('[open] ' .. vibook.buf.get_text()) end)
         vibook.on('note_save', function() vibook.buf.set_text('[save] ' .. vibook.buf.get_text()) end)",
    );
    let ws = ws("BODY");
    let path = PathBuf::from("/vault/a.md");
    assert_eq!(
        modules.event(&ws, &Event::NoteOpen(path.clone())),
        vec![text("[open] BODY")]
    );
    assert_eq!(
        modules.event(&ws, &Event::NoteSave(path)),
        vec![text("[save] BODY")]
    );
}

#[test]
fn vault_changed_feeds_vault_queries() {
    let mut modules = started("s6", 
        "vibook.command('peek', function()
           local back = vibook.vault.backlinks('a.md')
           vibook.buf.set_text(#vibook.vault.notes() .. ',' .. vibook.vault.tags()[1] .. ',' .. #back)
         end)",
    );
    let ws = ws("");
    let changed = |backlinks| Event::VaultChanged {
        notes: vec![String::from("a.md"), String::from("b.md")],
        tags: vec![String::from("idea")],
        backlinks,
    };
    let linked = HashMap::from([(String::from("a.md"), vec![String::from("b.md")])]);
    assert_eq!(modules.event(&ws, &changed(linked)), Vec::new());
    assert_eq!(
        modules.command(&ws, "peek", ""),
        Some(vec![text("2,idea,1")])
    );
    modules.event(&ws, &changed(HashMap::new()));
    assert_eq!(
        modules.command(&ws, "peek", ""),
        Some(vec![text("2,idea,0")])
    );
}

#[test]
fn vault_create_becomes_new_command() {
    let mut modules = started(
        "s7",
        "vibook.command('mk', function() vibook.vault.create('idea') end)",
    );
    assert_eq!(
        modules.command(&ws(""), "mk", ""),
        Some(vec![Request::Command(String::from("new idea"))])
    );
}

#[test]
fn hook_open_is_drained_not_leaked() {
    let mut modules = started(
        "s8",
        "vibook.on('note_open', function() vibook.open('b.md') end)
         vibook.command('noop', function() end)",
    );
    let ws = ws("");
    assert_eq!(
        modules.event(&ws, &Event::NoteOpen(PathBuf::from("/vault/a.md"))),
        vec![Request::Open(PathBuf::from("/vault/b.md"))]
    );
    assert_eq!(modules.command(&ws, "noop", ""), Some(Vec::new()));
}

#[test]
fn config_intents_do_not_leak_into_first_call() {
    let mut modules = started(
        "s9",
        "vibook.open('x.md') vibook.open_external('y.rs') vibook.vault.create('z')
         vibook.command('noop', function() end)",
    );
    assert_eq!(modules.command(&ws(""), "noop", ""), Some(Vec::new()));
}

#[test]
fn follow_takes_one_intent_external_first() {
    let mut modules = started(
        "s10",
        "vibook.on('follow_link', function(t)
           if t == 'both' then vibook.open('a.md') vibook.open_external('x.rs') vibook.open_external('y.rs') end
           if t == 'opens' then vibook.open('a.md') vibook.open('b.md') end
         end)",
    );
    assert_eq!(
        follow(&mut modules, "both"),
        vec![Request::OpenExternal(String::from("y.rs"))]
    );
    assert_eq!(
        follow(&mut modules, "opens"),
        vec![Request::Open(PathBuf::from("/vault/b.md"))]
    );
}
