mod common;

use std::fs;
use std::path::PathBuf;

use v_ide_editor::document::Document;
use v_ide_editor::workspace::Workspace;
use v_ide_keys::{Code, Key, Mods};
use v_ide_tui::app::App;
use v_ide_tui::view::View;

fn temp_vault(name: &str) -> PathBuf {
    common::scratch(&format!("buf-{name}"))
}

fn pool_buffer(root: &std::path::Path, rel: &str) -> Document {
    let mut doc = Document::open(root.join(rel)).unwrap();
    doc.name = rel.to_string();
    doc
}

#[test]
fn open_rel_dedups_to_existing_buffer_without_reload() {
    let root = temp_vault("dedup");
    fs::write(root.join("a.md"), "A body").unwrap();
    fs::write(root.join("b.md"), "B body").unwrap();

    let mut app = App::new(common::named_doc("", ""));
    common::open_vault(&mut app, &root);
    app.open_rel("a.md");
    app.open_rel("b.md");
    assert_eq!(app.workspace.current().name, "b.md");
    assert_eq!(
        app.workspace.pool().len(),
        1,
        "a.md is pooled, no empty buffer leaks in"
    );

    app.workspace.current_mut().editor.handle_keys("ix<Esc>");
    let dirty = app.workspace.current().editor.text();
    assert_ne!(dirty, "B body");

    app.open_rel("a.md");
    assert_eq!(
        app.workspace.current().name,
        "a.md",
        "open of pooled file switches to it"
    );

    app.open_rel("b.md");
    assert_eq!(
        app.workspace.current().name,
        "b.md",
        "switching back reaches the pooled b.md"
    );
    assert_eq!(
        app.workspace.current().editor.text(),
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

    let mut app = App::new(common::named_doc("", ""));
    common::open_vault(&mut app, &root);
    app.open_rel("a.md");
    app.workspace
        .current_mut()
        .editor
        .set_cursor(v_ide_text::Position { line: 1, col: 2 });
    let cursor = app.workspace.current().editor.cursor();
    let pool_len = app.workspace.pool().len();

    app.open_rel("a.md");
    assert_eq!(
        app.workspace.current().editor.cursor(),
        cursor,
        "reopening active keeps cursor"
    );
    assert_eq!(
        app.workspace.pool().len(),
        pool_len,
        "no self-stash on no-op open"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn ctrl_o_i_navigate_through_buffers_without_reload() {
    let root = temp_vault("hist-pool");
    fs::write(root.join("a.md"), "A body").unwrap();
    fs::write(root.join("b.md"), "B body").unwrap();
    let mut app = three_buffer_app(&root);
    app.open_rel("a.md");
    app.open_rel("b.md");
    app.workspace.current_mut().editor.handle_keys("ix<Esc>");
    let dirty = app.workspace.current().editor.text();
    assert_ne!(dirty, "B body");

    v_ide_tui::input::global_nav(&mut app, v_ide_keys::Key::ctrl('o'));
    assert_eq!(
        app.workspace.current().name,
        "a.md",
        "Ctrl-o switches back through the pool"
    );
    assert!(
        app.workspace
            .pool()
            .iter()
            .any(|b| b.doc.name == "b.md" && b.doc.editor.text() == dirty),
        "b.md is stashed dirty, not written or reloaded"
    );

    v_ide_tui::input::global_nav(&mut app, v_ide_keys::Key::ctrl('i'));
    assert_eq!(
        app.workspace.current().name,
        "b.md",
        "Ctrl-i switches forward through the pool"
    );
    assert_eq!(
        app.workspace.current().editor.text(),
        dirty,
        "forward navigation reuses the buffer, never reloads from disk"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn buffer_pool_is_global_across_split_focus_changes() {
    let root = temp_vault("pool-splits");
    fs::write(root.join("a.md"), "A").unwrap();
    fs::write(root.join("b.md"), "B").unwrap();
    fs::write(root.join("c.md"), "C").unwrap();
    let mut app = three_buffer_app(&root);
    app.open_rel("a.md");
    app.open_rel("b.md");
    let pool: Vec<String> = app
        .workspace
        .pool()
        .iter()
        .map(|b| b.doc.name.clone())
        .collect();
    assert_eq!(pool, vec!["a.md"], "pool holds the previous buffer");

    app.vsplit(Some("c.md"));
    let after_split: Vec<String> = app
        .workspace
        .pool()
        .iter()
        .map(|b| b.doc.name.clone())
        .collect();
    assert_eq!(
        after_split, pool,
        "splitting does not move the pool into the new pane"
    );

    app.focus_next();
    let after_focus: Vec<String> = app
        .workspace
        .pool()
        .iter()
        .map(|b| b.doc.name.clone())
        .collect();
    assert_eq!(
        after_focus, pool,
        "the pool stays global on App when focus moves between panes"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn buffers_command_opens_list_selecting_previous_buffer() {
    let root = temp_vault("blist-open");
    fs::write(root.join("a.md"), "A").unwrap();
    fs::write(root.join("b.md"), "B").unwrap();
    let mut app = three_buffer_app(&root);
    app.open_rel("b.md");
    app.open_rel("a.md");

    v_ide_tui::command_line::run(&mut app, "buffers");
    let bl = app.ui.buffer_list.as_ref().expect("buffer list opens");
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

    v_ide_tui::command_line::run(&mut app, "buffers");
    v_ide_tui::input::buffer_list_key(&mut app, Key::new(Code::Enter, Mods::NONE));
    assert_eq!(
        app.workspace.current().name,
        "b.md",
        "Enter switches to the selected buffer"
    );
    assert!(app.ui.buffer_list.is_none(), "Enter closes the list");

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

    v_ide_tui::command_line::run(&mut app, "buffers");
    v_ide_tui::input::buffer_list_key(&mut app, Key::ctrl('d'));
    assert!(app.ui.buffer_list.is_some(), "Ctrl-d keeps the list open");
    let entries = app.ui.buffer_list.as_ref().unwrap().filtered();
    assert!(
        entries.iter().all(|e| e.path != "b.md"),
        "selected (previous) buffer is closed"
    );
    assert_eq!(
        app.workspace.current().name,
        "a.md",
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

    v_ide_tui::command_line::run(&mut app, "buffers");
    v_ide_tui::input::buffer_list_key(&mut app, Key::new(Code::Esc, Mods::NONE));
    assert!(app.ui.buffer_list.is_none());

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn quit_is_blocked_when_a_pooled_buffer_is_dirty() {
    let root = temp_vault("quit-dirty-pool");
    fs::write(root.join("a.md"), "A").unwrap();
    fs::write(root.join("b.md"), "B").unwrap();
    let mut app = three_buffer_app(&root);
    app.open_rel("b.md");
    app.workspace.current_mut().editor.handle_keys("ix<Esc>");
    app.open_rel("a.md");
    assert!(!app.workspace.current().is_dirty(), "active a.md is clean");
    assert!(app.workspace.pool().iter().any(|b| b.doc.is_dirty()));

    v_ide_tui::command_line::run(&mut app, "q");
    assert!(!app.quit, "q is blocked by a dirty pooled buffer");
    assert!(app.ui.message.contains("unsaved"));

    v_ide_tui::command_line::run(&mut app, "q!");
    assert!(app.quit, "q! force-quits regardless of dirty buffers");

    let _ = fs::remove_dir_all(&root);
}

fn three_buffer_app(root: &std::path::Path) -> App {
    let mut app = App::new(common::named_doc("", ""));
    common::open_vault(&mut app, root);
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
    assert_eq!(app.workspace.current().name, "a.md");
    assert_eq!(app.workspace.pool().len(), 1, "b.md is pooled");

    app.close_buffer(1);
    assert_eq!(app.workspace.current().name, "a.md", "active untouched");
    assert!(app.workspace.pool().is_empty(), "pooled buffer removed");

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn close_dirty_buffer_is_blocked() {
    let root = temp_vault("close-dirty");
    fs::write(root.join("a.md"), "A").unwrap();
    let mut app = three_buffer_app(&root);
    app.open_rel("a.md");
    app.workspace.current_mut().editor.handle_keys("ix<Esc>");
    assert!(app.workspace.current().is_dirty());

    app.close_buffer(0);
    assert!(
        app.ui.message.contains("unsaved"),
        "dirty close is blocked with a message"
    );
    assert_eq!(app.workspace.current().name, "a.md", "buffer stays");

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
    assert_eq!(app.workspace.current().name, "a.md");

    app.close_buffer(0);
    assert_eq!(
        app.workspace.current().name,
        "b.md",
        "closing active switches to MRU"
    );
    assert!(
        app.workspace.pool().is_empty(),
        "old active is gone, not re-pooled"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn close_last_buffer_opens_picker() {
    let root = temp_vault("close-last");
    fs::write(root.join("a.md"), "A").unwrap();
    let mut app = three_buffer_app(&root);
    app.open_rel("a.md");
    assert!(app.workspace.pool().is_empty());

    app.close_buffer(0);
    assert!(
        app.ui.picker.is_some(),
        "closing last buffer opens the picker"
    );
    assert!(
        app.workspace.current().name.is_empty(),
        "no active named buffer remains"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn switch_to_buffer_preserves_active_state_and_mru() {
    let root = temp_vault("switch");
    fs::write(root.join("a.md"), "hello").unwrap();
    fs::write(root.join("b.md"), "target body").unwrap();

    let mut doc = Document::open(root.join("a.md")).unwrap();
    doc.name = "a.md".into();
    let mut app = App::new(doc);
    common::open_vault(&mut app, &root);
    app.ui.read.view = View::Edit;
    app.workspace.current_mut().editor.handle_keys("ix<Esc>");
    app.ui.read.top = 7;
    let dirty_text = app.workspace.current().editor.text();
    let cursor = app.workspace.current().editor.cursor();
    assert!(app.workspace.current().is_dirty());

    app.workspace.add(pool_buffer(&root, "b.md"));

    app.switch_to_buffer(0);

    assert_eq!(
        app.workspace.current().name,
        "b.md",
        "active buffer is now the switch target"
    );
    assert_eq!(app.workspace.current().editor.text(), "target body");
    assert_eq!(app.workspace.pool().len(), 1);

    let stashed = &app.workspace.pool()[0];
    assert_eq!(
        stashed.doc.name, "a.md",
        "previous active stashed at MRU front"
    );
    assert_eq!(
        stashed.doc.editor.text(),
        dirty_text,
        "dirty text preserved"
    );
    assert!(stashed.doc.is_dirty(), "dirty flag preserved");
    assert_eq!(stashed.doc.editor.cursor(), cursor, "cursor preserved");
    let state = &app.read_states[&stashed.id];
    assert_eq!(state.view, View::Edit, "view preserved");
    assert_eq!(state.top, 7, "viewport top preserved");

    app.switch_to_buffer(0);
    assert_eq!(
        app.workspace.current().name,
        "a.md",
        "switching back restores stashed buffer"
    );
    assert_eq!(
        app.workspace.pool()[0].doc.name,
        "b.md",
        "MRU: prev active moves to front"
    );
    app.workspace.current_mut().editor.handle_keys("u");
    assert_eq!(
        app.workspace.current().editor.text(),
        "hello",
        "undo history preserved across switch"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn doc_mut_reaches_current_and_pooled_documents() {
    let mut first = Document::new("a");
    first.name = String::from("a.md");
    let mut ws = Workspace::new(first);
    let pooled = ws.current_id();
    let mut second = Document::new("b");
    second.name = String::from("b.md");
    ws.open(second);
    let current = ws.current_id();
    assert_eq!(
        ws.doc_mut(pooled).map(|d| d.name.clone()).as_deref(),
        Some("a.md")
    );
    assert_eq!(
        ws.doc_mut(current).map(|d| d.name.clone()).as_deref(),
        Some("b.md")
    );
}
