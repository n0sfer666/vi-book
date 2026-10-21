mod common;

use std::fs;
use std::path::{Path, PathBuf};

use common::vault::{strings, ws_at};
use v_ide_editor::module::{Event, Modules, Request};
use v_ide_editor::workspace::Workspace;
use v_ide_vault::index::{Index, Resolution};
use v_ide_vault::module::VaultModule;
use v_ide_vault::names::Names;
use v_ide_vault::note::Note;
use v_ide_vault::scan::walk_paths;

fn temp_vault(name: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("vault-names-{name}"));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("sub")).unwrap();
    fs::write(root.join("a.md"), "# A\n").unwrap();
    fs::write(root.join("sub/b.md"), "plain\n").unwrap();
    root
}

fn started(ws: &Workspace) -> Modules {
    let mut modules = Modules::new(vec![VaultModule::registration()]);
    modules.startup(ws);
    modules
}

fn vault(modules: &Modules) -> &VaultModule {
    modules.get::<VaultModule>().unwrap()
}

fn backlinks(modules: &mut Modules, root: &Path) -> Option<Vec<Request>> {
    modules.command(&ws_at(root, Some("a.md")), "backlinks", "")
}

fn linked_from(rows: &[&str]) -> Option<Vec<Request>> {
    Some(vec![Request::List {
        title: String::from("backlinks"),
        rows: strings(rows),
        command: String::from("e"),
    }])
}

fn assert_vault_error(requests: Option<Vec<Request>>) {
    let failed = requests.unwrap();
    assert!(
        matches!(failed.as_slice(), [Request::Message(m)] if m.starts_with("vault: ")),
        "{failed:?}"
    );
}

#[test]
fn names_list_paths_and_resolve_by_stem() {
    let names = Names::from_paths(strings(&["b/x.md", "a.md", "c/x.md"]));
    assert_eq!(
        names.paths(),
        strings(&["b/x.md", "a.md", "c/x.md"]).as_slice()
    );
    assert_eq!(names.resolve("a"), Resolution::One(String::from("a.md")));
    assert_eq!(
        names.resolve("x"),
        Resolution::Many(strings(&["b/x.md", "c/x.md"]))
    );
    assert_eq!(names.resolve("ghost"), Resolution::None);
}

#[test]
fn index_exposes_its_names() {
    let index = Index::from_notes(vec![
        Note::from_source("z.md", "[[a]]"),
        Note::from_source("a.md", "x"),
    ]);
    assert_eq!(index.names().paths(), strings(&["z.md", "a.md"]).as_slice());
    assert_eq!(index.names().resolve("z"), index.resolve("z"));
}

#[test]
fn walk_paths_lists_relative_sorted_paths() {
    let root = temp_vault("walk");
    fs::create_dir_all(root.join(".h")).unwrap();
    fs::write(root.join(".h/c.md"), "# C\n").unwrap();
    fs::write(root.join("sub/skip.txt"), "x").unwrap();
    let plain = walk_paths(&root, false).unwrap();
    let hidden = walk_paths(&root, true).unwrap();
    assert_eq!(plain, strings(&["a.md", "sub/b.md"]));
    assert_eq!(hidden, strings(&[".h/c.md", "a.md", "sub/b.md"]));
}

#[test]
fn picker_does_not_parse_notes() {
    let root = temp_vault("picker");
    let mut modules = started(&ws_at(&root, None));
    assert_eq!(
        vault(&modules).items().unwrap(),
        strings(&["a.md", "sub/b.md"])
    );
    fs::write(root.join("sub/b.md"), "see [[a]]\n").unwrap();
    assert_eq!(backlinks(&mut modules, &root), linked_from(&["sub/b.md"]));
}

#[test]
fn names_resolve_without_parsing_notes() {
    let root = temp_vault("resolve");
    let mut modules = started(&ws_at(&root, None));
    let found = vault(&modules).names().unwrap().resolve("b");
    assert_eq!(found, Resolution::One(String::from("sub/b.md")));
    fs::write(root.join("sub/b.md"), "see [[a]]\n").unwrap();
    assert_eq!(backlinks(&mut modules, &root), linked_from(&["sub/b.md"]));
}

#[test]
fn note_create_with_names_only_resets_silently() {
    let root = temp_vault("create");
    let ws = ws_at(&root, None);
    let mut modules = started(&ws);
    assert_eq!(vault(&modules).items().unwrap().len(), 2);
    let abs = root.join("fresh.md");
    fs::write(&abs, "# F\n").unwrap();
    assert_eq!(modules.event(&ws, &Event::NoteCreate(abs)), Vec::new());
    assert_eq!(
        vault(&modules).items().unwrap(),
        strings(&["a.md", "fresh.md", "sub/b.md"])
    );
}

#[test]
fn scan_hidden_with_names_only_resets_silently() {
    let root = temp_vault("hidden");
    let ws = ws_at(&root, None);
    let mut modules = started(&ws);
    assert_eq!(vault(&modules).items().unwrap().len(), 2);
    let on = modules.command(&ws, "scan-hidden", "on");
    fs::create_dir_all(root.join(".h")).unwrap();
    fs::write(root.join(".h/x.md"), "# X\n").unwrap();
    let message = Request::Message(String::from("scan hidden folders: on"));
    assert_eq!(on, Some(vec![message]));
    assert_eq!(
        vault(&modules).items().unwrap(),
        strings(&[".h/x.md", "a.md", "sub/b.md"])
    );
}

#[test]
fn building_facts_refreshes_names() {
    let root = temp_vault("refresh");
    let mut modules = started(&ws_at(&root, None));
    assert_eq!(vault(&modules).items().unwrap().len(), 2);
    fs::write(root.join("late.md"), "[[a]]\n").unwrap();
    assert_eq!(backlinks(&mut modules, &root), linked_from(&["late.md"]));
    assert_eq!(
        vault(&modules).items().unwrap(),
        strings(&["a.md", "late.md", "sub/b.md"])
    );
}

#[test]
fn unreadable_note_keeps_picker_and_fails_facts() {
    let root = temp_vault("utf8");
    fs::write(root.join("bad.md"), [0xff, 0xfe, 0x00]).unwrap();
    let mut modules = started(&ws_at(&root, None));
    assert_eq!(
        vault(&modules).items().unwrap(),
        strings(&["a.md", "bad.md", "sub/b.md"])
    );
    let found = vault(&modules).names().unwrap().resolve("b");
    assert_eq!(found, Resolution::One(String::from("sub/b.md")));
    assert_vault_error(backlinks(&mut modules, &root));
}

#[test]
fn failed_reindex_falls_back_to_names() {
    let root = temp_vault("stale");
    let ws = ws_at(&root, None);
    let mut modules = started(&ws);
    assert_eq!(vault(&modules).index().unwrap().notes().len(), 2);
    let abs = root.join("fresh.md");
    fs::write(&abs, [0xff, 0xfe, 0x00]).unwrap();
    assert_eq!(modules.event(&ws, &Event::NoteCreate(abs)), Vec::new());
    let found = vault(&modules).names().unwrap().resolve("fresh");
    assert_eq!(found, Resolution::One(String::from("fresh.md")));
    assert_vault_error(backlinks(&mut modules, &root));
}
