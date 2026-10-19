mod common;
#[path = "module_activation/fakes.rs"]
mod fakes;

use std::fs;
use std::path::PathBuf;

use v_ide_editor::document::Document;
use v_ide_editor::module::{Event, Modules, Request};
use v_ide_editor::workspace::Workspace;
use v_ide_lua::module::LuaModule;
use v_ide_tui::app::App;

#[test]
fn lua_stays_inactive_without_config() {
    let mut app = App::new(common::named_doc("x", "a.md"));
    common::open_vault(&mut app, &common::temp_root("lazy-none"));
    v_ide_tui::lua_bridge::load_config_from(&mut app, None);
    assert!(app.modules.get::<LuaModule>().is_none());
}

#[test]
fn follow_link_activates_lua() {
    let mut modules = Modules::new(vec![LuaModule::registration()]);
    let mut ws = Workspace::new(Document::new(""));
    ws.set_cwd(PathBuf::from("/vault"));
    assert_eq!(
        modules.event(&ws, &Event::FollowLink(String::from("b.md"))),
        vec![Request::Open(PathBuf::from("/vault/b.md"))]
    );
    assert!(modules.get::<LuaModule>().is_some());
}

#[test]
fn opening_markdown_note_activates_filetype_modules() {
    let root = common::temp_root("lazy-ft");
    fs::write(root.join("a.md"), "# A\n").unwrap();
    let mut app = App::new(common::named_doc("", ""));
    app.workspace.set_cwd(root.clone());
    app.modules = fakes::modules();
    app.open_rel("a.md");
    let _ = fs::remove_dir_all(&root);
    assert_eq!(app.ui.message, "md up");
}
