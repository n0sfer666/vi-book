use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use vi_book::lua::host::PluginHost;
use vi_book::ui::app::App;
use vi_book::vim::editor::Editor;

fn ctrl(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
}

fn host_with(script: &str) -> PluginHost {
    let host = PluginHost::new().unwrap();
    host.load(script).unwrap();
    host
}

#[test]
fn normal_keymap_runs_plugin_command() {
    let host = host_with(
        "vibook.command('shout', function() vibook.buf.set_text('SHOUTED') end)\n\
         vibook.keymap('n', '<C-s>', ':shout')",
    );
    let mut app = App::new(Editor::new("hi"), "a.md".into());
    app.host = Some(host);
    app.install_keymaps();

    app.handle(ctrl('s'));
    assert_eq!(app.editor.text(), "SHOUTED");
}

#[test]
fn normal_keymap_feeds_editor_keys() {
    let host = host_with("vibook.keymap('n', '<C-d>', 'dd')");
    let mut app = App::new(Editor::new("one\ntwo"), "a.md".into());
    app.host = Some(host);
    app.install_keymaps();

    app.handle(ctrl('d'));
    assert_eq!(app.editor.text(), "two");
}

#[test]
fn keymaps_without_host_do_nothing() {
    let mut app = App::new(Editor::new("hi"), "a.md".into());
    app.install_keymaps();
    app.handle(ctrl('s'));
    assert_eq!(app.editor.text(), "hi");
}
