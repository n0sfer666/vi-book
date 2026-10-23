mod common;

use std::fs;

use common::vault::strings;
use common::watch::{
    alive, announced, announcements, facts_match, names_match, quiet, settle, temp_vault, vault,
    with_facts, with_names,
};

#[test]
fn folder_rename_with_facts_reindexes() {
    let root = temp_vault("dir-facts");
    let (ws, mut modules) = with_facts(&root);
    fs::rename(root.join("sub"), root.join("moved")).unwrap();
    let ticks = settle(&ws, &mut modules, facts_match(&root));
    assert!(announced(&ticks), "{ticks:?}");
    assert_eq!(
        vault(&modules).items().unwrap(),
        strings(&["a.md", "moved/b.md"])
    );
}

#[test]
fn folder_rename_with_names_only_refreshes_names_silently() {
    let root = temp_vault("dir-names");
    let (ws, mut modules) = with_names(&root);
    fs::rename(root.join("sub"), root.join("moved")).unwrap();
    let ticks = settle(&ws, &mut modules, names_match(&root));
    assert!(ticks.iter().all(Vec::is_empty), "{ticks:?}");
}

#[test]
fn folder_removal_with_facts_drops_its_notes() {
    let root = temp_vault("dir-remove");
    let (ws, mut modules) = with_facts(&root);
    fs::remove_dir_all(root.join("sub")).unwrap();
    let ticks = settle(&ws, &mut modules, facts_match(&root));
    assert!(announced(&ticks), "{ticks:?}");
    assert_eq!(vault(&modules).items().unwrap(), strings(&["a.md"]));
}

#[test]
fn md_named_folder_rename_moves_its_notes() {
    let root = temp_vault("dir-md");
    fs::create_dir_all(root.join("x.md")).unwrap();
    fs::write(root.join("x.md/y.md"), "# Y\n").unwrap();
    let (ws, mut modules) = with_facts(&root);
    fs::rename(root.join("x.md"), root.join("moved.md")).unwrap();
    let ticks = settle(&ws, &mut modules, facts_match(&root));
    assert!(announced(&ticks), "{ticks:?}");
    assert_eq!(
        vault(&modules).items().unwrap(),
        strings(&["a.md", "moved.md/y.md", "sub/b.md"])
    );
}

#[test]
fn changes_outside_scope_are_silent() {
    let root = temp_vault("dir-noise");
    let (ws, mut modules) = with_facts(&root);
    fs::create_dir_all(root.join(".hidden")).unwrap();
    fs::write(root.join(".hidden/x.md"), "# X\n").unwrap();
    fs::write(root.join("t.txt"), "text\n").unwrap();
    fs::create_dir_all(root.join("target/out")).unwrap();
    fs::write(root.join("target/out/y.o"), "obj\n").unwrap();
    assert_eq!(quiet(&ws, &mut modules), Vec::new());
    assert_eq!(
        vault(&modules).items().unwrap(),
        strings(&["a.md", "sub/b.md"])
    );
    assert_eq!(announcements(&alive(&root, &ws, &mut modules)), 1);
}
