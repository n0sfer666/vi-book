mod common;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use v_ide_tui::app::App;
use v_ide_tui::lua_bridge;

fn ctrl(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
}

#[test]
fn normal_keymap_runs_plugin_command() {
    let mut app = common::lua_app(
        common::named_doc("hi", "a.md"),
        "keymap-cmd",
        "vibook.command('shout', function() vibook.buf.set_text('SHOUTED') end)\n\
         vibook.keymap('n', '<C-s>', ':shout')",
    );
    app.handle(ctrl('s'));
    assert_eq!(app.workspace.current().editor.text(), "SHOUTED");
}

#[test]
fn normal_keymap_feeds_editor_keys() {
    let mut app = common::lua_app(
        common::named_doc("one\ntwo", "a.md"),
        "keymap-keys",
        "vibook.keymap('n', '<C-d>', 'dd')",
    );
    app.handle(ctrl('d'));
    assert_eq!(app.workspace.current().editor.text(), "two");
}

#[test]
fn keymaps_without_lua_module_do_nothing() {
    let mut app = App::new(common::named_doc("hi", "a.md"));
    lua_bridge::load_config_from(&mut app, None);
    app.handle(ctrl('s'));
    assert_eq!(app.workspace.current().editor.text(), "hi");
}
