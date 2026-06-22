use std::fs;
use std::path::PathBuf;

use vi_book::ui::app::App;
use vi_book::ui::command_line;
use vi_book::ui::input;
use vi_book::ui::panel::Panel;
use vi_book::ui::session::Session;
use vi_book::Editor;
use vi_book::Key;

fn temp_vault(name: &str) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("vi-book-backlinks-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

fn app_for(root: &std::path::Path) -> App {
    let (session, _) = Session::open(Some(root.to_str().unwrap())).unwrap();
    let mut app = App::new(Editor::new(""), String::new());
    app.session = session;
    app
}

#[test]
fn empty_backlinks_shows_message_and_no_panel() {
    let root = temp_vault("empty");
    fs::write(root.join("a.md"), "# A\n").unwrap();
    let mut app = app_for(&root);
    app.open_rel("a.md");
    command_line::run(&mut app, "backlinks");
    let panel = app.panel.is_some();
    let message = app.message.clone();
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
    let is_backlinks = matches!(app.panel, Some(Panel::Backlinks { .. }));
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
    let path = app.path.clone();
    let panel = app.panel.is_some();
    let focus = app.panel_focus;
    let _ = fs::remove_dir_all(&root);
    assert_eq!(path, "b.md", "Enter must open the backlinked file");
    assert!(!panel, "Enter on a backlink must close the panel");
    assert!(!focus, "panel focus must be released after opening");
}
