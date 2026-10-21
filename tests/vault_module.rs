mod common;

use std::fs;
use std::path::{Path, PathBuf};

use common::vault::{indexed, is_changed, items, msg, strings, ws_at};
use v_ide_editor::module::{Event, Modules, Request};
use v_ide_editor::workspace::Workspace;
use v_ide_vault::module::VaultModule;

fn temp_vault(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "vi-book-vault-module-{}-{name}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

fn started(ws: &Workspace) -> Modules {
    let mut modules = Modules::new(vec![VaultModule::registration()]);
    assert_eq!(modules.startup(ws), Vec::new());
    modules
}

fn seed(root: &Path) {
    fs::create_dir_all(root.join("sub")).unwrap();
    fs::create_dir_all(root.join(".hidden")).unwrap();
    fs::write(root.join("a.md"), "# A\n").unwrap();
    fs::write(root.join("sub/b.md"), "see [[a]]\n").unwrap();
    fs::write(root.join(".hidden/c.md"), "# C\n").unwrap();
}

#[test]
fn startup_indexes_cwd() {
    let root = temp_vault("startup");
    seed(&root);
    let modules = started(&ws_at(&root, None));
    let vault = modules.get::<VaultModule>().unwrap();
    assert_eq!(vault.root(), root.as_path());
    assert_eq!(vault.items().unwrap(), strings(&["a.md", "sub/b.md"]));
    assert_eq!(vault.index().unwrap().notes().len(), 2);
    assert!(!vault.items().unwrap().is_empty());
    assert!(!vault.scan_hidden());
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn empty_vault_is_empty() {
    let root = temp_vault("empty");
    let modules = started(&ws_at(&root, None));
    let items = modules.get::<VaultModule>().unwrap().items();
    assert!(items.is_ok_and(|items| items.is_empty()));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn pick_offers_notes_to_e() {
    let root = temp_vault("pick");
    seed(&root);
    let ws = ws_at(&root, None);
    let mut modules = started(&ws);
    let expected = Request::Pick {
        items: strings(&["a.md", "sub/b.md"]),
        command: String::from("e"),
    };
    assert_eq!(modules.command(&ws, "pick", ""), Some(vec![expected]));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn new_creates_note_reindexes_and_opens() {
    let root = temp_vault("new");
    let ws = ws_at(&root, None);
    let mut modules = started(&ws);
    let abs = root.join("ideas/x.md");
    assert_eq!(indexed(&modules), 0);
    let requests = modules.command(&ws, "new", " ideas/x ");
    assert_eq!(
        requests,
        Some(vec![
            Request::Emit(Event::NoteCreate(abs.clone())),
            Request::Open(abs.clone()),
        ])
    );
    assert_eq!(fs::read_to_string(&abs).unwrap(), "# x\n\n");
    assert_eq!(items(&modules), Vec::<String>::new());
    let requests = modules.event(&ws, &Event::NoteCreate(abs));
    assert!(matches!(requests.as_slice(), [changed] if is_changed(changed)));
    assert_eq!(items(&modules), strings(&["ideas/x.md"]));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn new_keeps_existing_note() {
    let root = temp_vault("keep");
    fs::write(root.join("y.md"), "keep").unwrap();
    let ws = ws_at(&root, None);
    let mut modules = started(&ws);
    let requests = modules.command(&ws, "new", "y.md");
    assert_eq!(requests, Some(vec![Request::Open(root.join("y.md"))]));
    assert_eq!(fs::read_to_string(root.join("y.md")).unwrap(), "keep");
    assert_eq!(modules.command(&ws, "new", "  "), Some(Vec::new()));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn scan_hidden_command_toggles_and_reindexes() {
    let root = temp_vault("hidden");
    seed(&root);
    let ws = ws_at(&root, None);
    let mut modules = started(&ws);
    assert_eq!(indexed(&modules), 2);
    let on = modules.command(&ws, "scan-hidden", "on").unwrap();
    assert!(
        matches!(on.as_slice(), [c, m] if is_changed(c) && *m == msg("scan hidden folders: on")),
        "{on:?}"
    );
    assert_eq!(
        items(&modules),
        strings(&[".hidden/c.md", "a.md", "sub/b.md"])
    );
    let off = modules.command(&ws, "scan-hidden", "toggle").unwrap();
    assert!(is_changed(&off[0]));
    assert_eq!(off[1..], [msg("scan hidden folders: off")]);
    assert_eq!(items(&modules), strings(&["a.md", "sub/b.md"]));
    assert_eq!(
        modules.command(&ws, "scan-hidden", "maybe"),
        Some(vec![msg("scan-hidden: on|off|toggle")])
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn bridge_sets_scan_hidden_and_reindexes() {
    let root = temp_vault("bridge");
    seed(&root);
    let mut modules = started(&ws_at(&root, None));
    let vault = modules.get_mut::<VaultModule>().unwrap();
    assert_eq!(vault.index().unwrap().notes().len(), 2);
    vault.set_scan_hidden(true).unwrap();
    assert!(vault.scan_hidden());
    fs::write(root.join("d.md"), "# D\n").unwrap();
    assert_eq!(vault.items().unwrap().len(), 3);
    vault.reindex().unwrap();
    assert_eq!(vault.items().unwrap().len(), 4);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn backlinks_lists_linking_notes() {
    let root = temp_vault("backlinks");
    seed(&root);
    let ws = ws_at(&root, Some("a.md"));
    let mut modules = started(&ws);
    let expected = Request::List {
        title: String::from("backlinks"),
        rows: strings(&["sub/b.md"]),
        command: String::from("e"),
    };
    assert_eq!(modules.command(&ws, "backlinks", ""), Some(vec![expected]));
    let lonely = ws_at(&root, Some("sub/b.md"));
    assert_eq!(
        modules.command(&lonely, "backlinks", ""),
        Some(vec![msg("no notes link here")])
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn note_create_event_reindexes() {
    let root = temp_vault("note-create");
    seed(&root);
    let ws = ws_at(&root, None);
    let mut modules = started(&ws);
    assert_eq!(indexed(&modules), 2);
    let abs = root.join("fresh.md");
    fs::write(&abs, "# F\n").unwrap();
    let requests = modules.event(&ws, &Event::NoteCreate(abs));
    assert!(matches!(requests.as_slice(), [changed] if is_changed(changed)));
    assert_eq!(items(&modules), strings(&["a.md", "fresh.md", "sub/b.md"]));
    let _ = fs::remove_dir_all(&root);
}
