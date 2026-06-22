use std::fs;
use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
fn open_rel_dedups_to_existing_buffer_without_reload() {
    let root = temp_vault("dedup");
    fs::write(root.join("a.md"), "A body").unwrap();
    fs::write(root.join("b.md"), "B body").unwrap();
    let (session, _) = Session::open(Some(root.to_str().unwrap())).unwrap();

    let mut app = App::new(Editor::new(""), String::new());
    app.session = session;
    app.open_rel("a.md");
    app.open_rel("b.md");
    assert_eq!(app.path, "b.md");
    assert_eq!(
        app.buffers.len(),
        1,
        "a.md is pooled, no empty buffer leaks in"
    );

    app.editor.handle_keys("ix<Esc>");
    let dirty = app.editor.text();
    assert_ne!(dirty, "B body");

    app.open_rel("a.md");
    assert_eq!(app.path, "a.md", "open of pooled file switches to it");

    app.open_rel("b.md");
    assert_eq!(app.path, "b.md", "switching back reaches the pooled b.md");
    assert_eq!(
        app.editor.text(),
        dirty,
        "dedup must not reload from disk — dirty edits survive"
    );
    assert_eq!(
        fs::read_to_string(root.join("b.md")).unwrap(),
        "B body",
        "disk untouched by switching"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn open_rel_of_active_file_is_a_noop() {
    let root = temp_vault("noop");
    fs::write(root.join("a.md"), "A body line\nsecond").unwrap();
    let (session, _) = Session::open(Some(root.to_str().unwrap())).unwrap();

    let mut app = App::new(Editor::new(""), String::new());
    app.session = session;
    app.open_rel("a.md");
    app.editor.set_cursor(vi_book::Position { line: 1, col: 2 });
    let cursor = app.editor.cursor();
    let pool_len = app.buffers.len();

    app.open_rel("a.md");
    assert_eq!(app.editor.cursor(), cursor, "reopening active keeps cursor");
    assert_eq!(app.buffers.len(), pool_len, "no self-stash on no-op open");

    let _ = fs::remove_dir_all(&root);
}

fn ctrl(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
}

#[test]
fn buffers_command_opens_list_selecting_previous_buffer() {
    let root = temp_vault("blist-open");
    fs::write(root.join("a.md"), "A").unwrap();
    fs::write(root.join("b.md"), "B").unwrap();
    let mut app = three_buffer_app(&root);
    app.open_rel("b.md");
    app.open_rel("a.md");

    vi_book::ui::command_line::run(&mut app, "buffers");
    let bl = app.buffer_list.as_ref().expect("buffer list opens");
    let entries = bl.filtered();
    assert_eq!(entries.len(), 2, "active + one pooled buffer");
    assert_eq!(entries[0].path, "a.md");
    assert!(entries[0].current, "active is marked current");
    assert_eq!(entries[1].path, "b.md");
    assert_eq!(
        bl.current().unwrap().path,
        "b.md",
        "cursor defaults to the previous (MRU) buffer"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn unnamed_buffer_is_not_listed() {
    let root = temp_vault("blist-unnamed");
    fs::write(root.join("a.md"), "A").unwrap();
    let mut app = three_buffer_app(&root);
    app.open_rel("a.md");

    let entries = app.named_buffers();
    assert_eq!(entries.len(), 1, "the empty startup buffer is excluded");
    assert_eq!(entries[0].path, "a.md");

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn buffer_list_enter_switches_and_closes() {
    let root = temp_vault("blist-enter");
    fs::write(root.join("a.md"), "A").unwrap();
    fs::write(root.join("b.md"), "B").unwrap();
    let mut app = three_buffer_app(&root);
    app.open_rel("b.md");
    app.open_rel("a.md");

    vi_book::ui::command_line::run(&mut app, "buffers");
    vi_book::ui::input::buffer_list_key(&mut app, KeyEvent::from(KeyCode::Enter));
    assert_eq!(app.path, "b.md", "Enter switches to the selected buffer");
    assert!(app.buffer_list.is_none(), "Enter closes the list");

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn buffer_list_ctrl_d_closes_buffer_and_stays_open() {
    let root = temp_vault("blist-ctrld");
    fs::write(root.join("a.md"), "A").unwrap();
    fs::write(root.join("b.md"), "B").unwrap();
    fs::write(root.join("c.md"), "C").unwrap();
    let mut app = three_buffer_app(&root);
    app.open_rel("c.md");
    app.open_rel("b.md");
    app.open_rel("a.md");

    vi_book::ui::command_line::run(&mut app, "buffers");
    vi_book::ui::input::buffer_list_key(&mut app, ctrl('d'));
    assert!(app.buffer_list.is_some(), "Ctrl-d keeps the list open");
    let entries = app.buffer_list.as_ref().unwrap().filtered();
    assert!(
        entries.iter().all(|e| e.path != "b.md"),
        "selected (previous) buffer is closed"
    );
    assert_eq!(
        app.path, "a.md",
        "closing a pooled buffer leaves active alone"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn buffer_list_esc_closes() {
    let root = temp_vault("blist-esc");
    fs::write(root.join("a.md"), "A").unwrap();
    let mut app = three_buffer_app(&root);
    app.open_rel("a.md");

    vi_book::ui::command_line::run(&mut app, "buffers");
    vi_book::ui::input::buffer_list_key(&mut app, KeyEvent::from(KeyCode::Esc));
    assert!(app.buffer_list.is_none());

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn quit_is_blocked_when_a_pooled_buffer_is_dirty() {
    let root = temp_vault("quit-dirty-pool");
    fs::write(root.join("a.md"), "A").unwrap();
    fs::write(root.join("b.md"), "B").unwrap();
    let mut app = three_buffer_app(&root);
    app.open_rel("b.md");
    app.editor.handle_keys("ix<Esc>");
    app.open_rel("a.md");
    assert_eq!(app.editor.text(), app.saved_text, "active a.md is clean");
    assert!(app.buffers.iter().any(|p| p.editor.text() != p.saved_text));

    vi_book::ui::command_line::run(&mut app, "q");
    assert!(!app.quit, "q is blocked by a dirty pooled buffer");
    assert!(app.message.contains("unsaved"));

    vi_book::ui::command_line::run(&mut app, "q!");
    assert!(app.quit, "q! force-quits regardless of dirty buffers");

    let _ = fs::remove_dir_all(&root);
}

fn three_buffer_app(root: &std::path::Path) -> App {
    let (session, _) = Session::open(Some(root.to_str().unwrap())).unwrap();
    let mut app = App::new(Editor::new(""), String::new());
    app.session = session;
    app
}

#[test]
fn close_clean_pool_buffer_removes_it() {
    let root = temp_vault("close-clean");
    fs::write(root.join("a.md"), "A").unwrap();
    fs::write(root.join("b.md"), "B").unwrap();
    let mut app = three_buffer_app(&root);
    app.open_rel("b.md");
    app.open_rel("a.md");
    assert_eq!(app.path, "a.md");
    assert_eq!(app.buffers.len(), 1, "b.md is pooled");

    app.close_buffer(1);
    assert_eq!(app.path, "a.md", "active untouched");
    assert!(app.buffers.is_empty(), "pooled buffer removed");

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn close_dirty_buffer_is_blocked() {
    let root = temp_vault("close-dirty");
    fs::write(root.join("a.md"), "A").unwrap();
    let mut app = three_buffer_app(&root);
    app.open_rel("a.md");
    app.editor.handle_keys("ix<Esc>");
    assert_ne!(app.editor.text(), app.saved_text);

    app.close_buffer(0);
    assert!(
        app.message.contains("unsaved"),
        "dirty close is blocked with a message"
    );
    assert_eq!(app.path, "a.md", "buffer stays");

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn close_active_switches_to_mru() {
    let root = temp_vault("close-active");
    fs::write(root.join("a.md"), "A").unwrap();
    fs::write(root.join("b.md"), "B").unwrap();
    let mut app = three_buffer_app(&root);
    app.open_rel("b.md");
    app.open_rel("a.md");
    assert_eq!(app.path, "a.md");

    app.close_buffer(0);
    assert_eq!(app.path, "b.md", "closing active switches to MRU");
    assert!(app.buffers.is_empty(), "old active is gone, not re-pooled");

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn close_last_buffer_opens_picker() {
    let root = temp_vault("close-last");
    fs::write(root.join("a.md"), "A").unwrap();
    let mut app = three_buffer_app(&root);
    app.open_rel("a.md");
    assert!(app.buffers.is_empty());

    app.close_buffer(0);
    assert!(app.picker.is_some(), "closing last buffer opens the picker");
    assert!(app.path.is_empty(), "no active named buffer remains");

    let _ = fs::remove_dir_all(&root);
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
