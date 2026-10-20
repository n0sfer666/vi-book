use std::fs;
use std::path::{Path, PathBuf};

use v_ide_editor::document::Document;
use v_ide_editor::module::{Event, Modules, Request};
use v_ide_editor::workspace::Workspace;
use v_ide_lua::module::LuaModule;
use v_ide_vault::module::VaultModule;

const PEEK: &str = "local function peek()
  return #vibook.vault.notes() .. ',' .. vibook.vault.tags()[1] .. ',' .. #vibook.vault.backlinks('a.md')
end
vibook.command('peek', function() vibook.buf.set_text(peek()) end)
vibook.on('follow_link', function() vibook.buf.set_text(peek()) end)";

fn temp_vault(name: &str) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("vi-book-lua-vault-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("sub")).unwrap();
    fs::write(root.join("a.md"), "# A\n").unwrap();
    fs::write(root.join("sub/b.md"), "see [[a]]\n").unwrap();
    fs::write(root.join("t.md"), "#idea\n").unwrap();
    root
}

fn ws_at(root: &Path) -> Workspace {
    let mut ws = Workspace::new(Document::new(""));
    ws.set_cwd(root.to_path_buf());
    ws
}

fn started(root: &Path, label: &str) -> Modules {
    let mut modules = Modules::new(vec![VaultModule::registration(), LuaModule::registration()]);
    let ws = ws_at(root);
    modules.startup(&ws);
    modules.activate(&ws, "lua");
    let path = root.join(format!("{label}.lua"));
    fs::write(&path, PEEK).unwrap();
    let lua = modules.get::<LuaModule>().expect("lua module active");
    assert_eq!(lua.load_config(Some((path, None))), Vec::<String>::new());
    modules
}

fn text(value: &str) -> Vec<Request> {
    vec![Request::SetText(String::from(value))]
}

#[test]
fn lua_queries_read_live_vault() {
    let root = temp_vault("live");
    let mut modules = started(&root, "live");
    let ws = ws_at(&root);
    let before = modules.command(&ws, "peek", "");
    let abs = root.join("c.md");
    fs::write(&abs, "back to [[a]]\n").unwrap();
    let changed = modules.event(&ws, &Event::NoteCreate(abs));
    let after = modules.command(&ws, "peek", "");
    let _ = fs::remove_dir_all(&root);
    assert_eq!(before, Some(text("3,idea,1")));
    assert!(matches!(
        changed.as_slice(),
        [Request::Emit(Event::VaultChanged { .. })]
    ));
    assert_eq!(after, Some(text("4,idea,2")));
}

#[test]
fn follow_hook_scans_vault_on_first_query() {
    let root = temp_vault("follow");
    let mut modules = started(&root, "follow");
    let ws = ws_at(&root);
    let requests = modules.event(&ws, &Event::FollowLink(String::from("a")));
    let _ = fs::remove_dir_all(&root);
    assert_eq!(requests, text("3,idea,1"));
}

#[test]
fn scan_error_raises_lua_error() {
    let root = temp_vault("error");
    let mut modules = started(&root, "error");
    let ws = ws_at(&root);
    let _ = fs::remove_dir_all(&root);
    let requests = modules.command(&ws, "peek", "").unwrap();
    assert!(
        matches!(requests.as_slice(), [Request::Message(m)]
            if m.starts_with("plugin error") && m.contains("vault: ")),
        "{requests:?}"
    );
}
