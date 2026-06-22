use std::fs;
use std::path::PathBuf;

use vi_book::ui::app::App;
use vi_book::ui::pane::Pane;
use vi_book::ui::session::Session;
use vi_book::ui::view::View;
use vi_book::vim::editor::Editor;

fn temp_vault(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("vi-book-buf-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

fn pool_buffer(root: &std::path::Path, rel: &str) -> Pane {
    let editor = Editor::open(root.join(rel)).unwrap();
    let saved_text = editor.text();
    Pane {
        editor,
        path: rel.to_string(),
        saved_text,
        top: 0,
        view: View::Read,
        read_cursor: (0, 0),
        read_pending_g: false,
        read_cache: None,
    }
}

#[test]
fn switch_to_buffer_preserves_active_state_and_mru() {
    let root = temp_vault("switch");
    fs::write(root.join("a.md"), "hello").unwrap();
    fs::write(root.join("b.md"), "target body").unwrap();
    let (session, _) = Session::open(Some(root.to_str().unwrap())).unwrap();

    let mut app = App::new(Editor::open(root.join("a.md")).unwrap(), "a.md".into());
    app.session = session;
    app.saved_text = app.editor.text();
    app.view = View::Edit;
    app.editor.handle_keys("ix<Esc>");
    app.top = 7;
    let dirty_text = app.editor.text();
    let cursor = app.editor.cursor();
    assert_ne!(dirty_text, app.saved_text);

    app.buffers.push(pool_buffer(&root, "b.md"));

    app.switch_to_buffer(0);

    assert_eq!(app.path, "b.md", "active buffer is now the switch target");
    assert_eq!(app.editor.text(), "target body");
    assert_eq!(app.buffers.len(), 1);

    let stashed = &app.buffers[0];
    assert_eq!(stashed.path, "a.md", "previous active stashed at MRU front");
    assert_eq!(stashed.editor.text(), dirty_text, "dirty text preserved");
    assert_ne!(
        stashed.editor.text(),
        stashed.saved_text,
        "dirty flag preserved"
    );
    assert_eq!(stashed.editor.cursor(), cursor, "cursor preserved");
    assert_eq!(stashed.view, View::Edit, "view preserved");
    assert_eq!(stashed.top, 7, "viewport top preserved");

    app.switch_to_buffer(0);
    assert_eq!(app.path, "a.md", "switching back restores stashed buffer");
    assert_eq!(
        app.buffers[0].path, "b.md",
        "MRU: prev active moves to front"
    );
    app.editor.handle_keys("u");
    assert_eq!(
        app.editor.text(),
        "hello",
        "undo history preserved across switch"
    );

    let _ = fs::remove_dir_all(&root);
}
