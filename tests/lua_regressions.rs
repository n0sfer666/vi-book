use std::fs;
use std::path::PathBuf;

use vi_book::lua::host::PluginHost;
use vi_book::lua::runtime::Runtime;
use vi_book::ui::app::App;
use vi_book::ui::session::Session;
use vi_book::vim::editor::Editor;

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
    let (session, _) = Session::open(Some(root.to_str().unwrap())).unwrap();

    let host = PluginHost::new().unwrap();
    host.load("vibook.on('note_open', function() vibook.buf.set_text('[open] ' .. vibook.buf.get_text()) end)")
        .unwrap();

    let mut app = App::new(Editor::new(""), String::new());
    app.session = session;
    app.host = Some(host);
    app.open_rel("a.md");

    let text = app.editor.text();
    let saved = app.saved_text.clone();
    let _ = fs::remove_dir_all(&root);
    assert_eq!(text, "[open] BODY", "note_open transform applied");
    assert_eq!(
        text, saved,
        "freshly opened note must not be reported dirty"
    );
}
