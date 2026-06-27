use std::fs;
use std::path::PathBuf;

use vi_book::ui::app::App;
use vi_book::ui::help;
use vi_book::ui::link_follow::follow_at_cursor;
use vi_book::ui::session::Session;
use vi_book::Editor;

fn temp_vault(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("vi-book-bundled-{}-{name}", std::process::id()));
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
fn bundled_doc_lookup_resolves_the_known_docs() {
    assert!(help::bundled_doc("manual.md").is_some());
    assert!(help::bundled_doc("developers.md").is_some());
    assert!(help::bundled_doc("plugin-development.md").is_some());
    assert!(help::bundled_doc("plugin-development.ru.md").is_some());
    assert!(help::bundled_doc("nope.md").is_none());
}

#[test]
fn enter_on_a_link_between_bundled_docs_opens_the_target() {
    let mut app = App::new(Editor::new(help::DEVELOPERS), "developers.md".into());
    let idx = help::DEVELOPERS.find("plugin-development.md").unwrap();
    app.editor.set_cursor_index(idx);
    follow_at_cursor(&mut app);
    assert_eq!(app.path, "plugin-development.md");
    assert!(app.editor.text().contains("plugin development"));
}

#[test]
fn links_in_ordinary_notes_are_not_shadowed_by_bundled_names() {
    let root = temp_vault("shadow");
    fs::write(root.join("manual.md"), "# Vault manual\n").unwrap();
    fs::write(root.join("note.md"), "see [m](manual.md)\n").unwrap();
    let mut app = app_for(&root);
    app.open_rel("note.md");
    app.editor.set_cursor_index(7);
    follow_at_cursor(&mut app);
    let path = app.path.clone();
    let text = app.editor.text();
    let _ = fs::remove_dir_all(&root);
    assert_eq!(path, "manual.md");
    assert_eq!(text, "# Vault manual\n");
}
