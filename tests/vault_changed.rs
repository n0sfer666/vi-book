use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use v_ide_editor::document::Document;
use v_ide_editor::module::{Event, Modules, Request};
use v_ide_editor::workspace::Workspace;
use v_ide_vault::module::VaultModule;

fn temp_vault(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "vi-book-vault-changed-{}-{name}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("sub")).unwrap();
    fs::write(root.join("a.md"), "# A\n").unwrap();
    fs::write(root.join("sub/b.md"), "see [[a]]\n").unwrap();
    fs::write(root.join("t.md"), "text #idea [[a]] [[a]] [[t]]\n").unwrap();
    root
}

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

#[test]
fn startup_announces_vault_snapshot() {
    let root = temp_vault("snapshot");
    let mut ws = Workspace::new(Document::new(""));
    ws.set_cwd(root.clone());
    let mut modules = Modules::new(vec![VaultModule::registration()]);
    let expected = Event::VaultChanged {
        notes: strings(&["a.md", "sub/b.md", "t.md"]),
        tags: strings(&["idea"]),
        backlinks: HashMap::from([(String::from("a.md"), strings(&["sub/b.md", "t.md"]))]),
    };
    let requests = modules.startup(&ws);
    let _ = fs::remove_dir_all(&root);
    assert_eq!(requests, vec![Request::Emit(expected)]);
}
