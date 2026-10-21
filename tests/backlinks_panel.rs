mod common;

use std::fs;
use std::path::PathBuf;

use v_ide_keys::Key;
use v_ide_tui::app::App;
use v_ide_tui::command_line;
use v_ide_tui::input;
use v_ide_tui::panel::Panel;

fn temp_vault(name: &str) -> PathBuf {
    common::scratch(&format!("backlinks-{name}"))
}

fn app_for(root: &std::path::Path) -> App {
    let mut app = App::new(common::named_doc("", ""));
    common::open_vault(&mut app, root);
    app
}

#[test]
fn empty_backlinks_shows_message_and_no_panel() {
    let root = temp_vault("empty");
    fs::write(root.join("a.md"), "# A\n").unwrap();
    let mut app = app_for(&root);
    app.open_rel("a.md");
    command_line::run(&mut app, "backlinks");
    let panel = app.ui.panel.is_some();
    let message = app.ui.message.clone();
    let _ = fs::remove_dir_all(&root);
    assert!(!panel, "empty backlinks must not open a panel");
    assert!(!message.is_empty(), "empty backlinks must report a message");
}

#[test]
fn non_empty_backlinks_opens_panel() {
    let root = temp_vault("has");
    fs::write(root.join("a.md"), "# A\n").unwrap();
    fs::write(root.join("b.md"), "see [[a]]\n").unwrap();
    let mut app = app_for(&root);
    app.open_rel("a.md");
    command_line::run(&mut app, "backlinks");
    let is_backlinks =
        matches!(&app.ui.panel, Some(Panel::List(list)) if list.title() == "backlinks");
    let _ = fs::remove_dir_all(&root);
    assert!(is_backlinks, "backlinks with results must open the panel");
}

#[test]
fn enter_on_backlink_opens_file_and_closes_panel() {
    let root = temp_vault("enter");
    fs::write(root.join("a.md"), "# A\n").unwrap();
    fs::write(root.join("b.md"), "see [[a]]\n").unwrap();
    let mut app = app_for(&root);
    app.open_rel("a.md");
    command_line::run(&mut app, "backlinks");
    input::panel_key(&mut app, Some(Key::Enter));
    let path = app.workspace.current().name.clone();
    let panel = app.ui.panel.is_some();
    let focus = app.ui.panel_focus;
    let _ = fs::remove_dir_all(&root);
    assert_eq!(path, "b.md", "Enter must open the backlinked file");
    assert!(!panel, "Enter on a backlink must close the panel");
    assert!(!focus, "panel focus must be released after opening");
}

#[test]
fn backlinks_twice_closes_panel() {
    let root = temp_vault("twice");
    fs::write(root.join("a.md"), "# A\n").unwrap();
    fs::write(root.join("b.md"), "see [[a]]\n").unwrap();
    let mut app = app_for(&root);
    app.open_rel("a.md");
    command_line::run(&mut app, "backlinks");
    let opened = app.ui.panel.is_some();
    command_line::run(&mut app, "backlinks");
    let closed = app.ui.panel.is_none();
    let _ = fs::remove_dir_all(&root);
    assert!(opened && closed, "second :backlinks must close the panel");
}
