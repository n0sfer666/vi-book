mod common;

use std::fs;
use std::path::{Path, PathBuf};

use common::scratch;
use common::vault::{is_changed, strings, ws_at};
use v_ide_editor::module::{Event, Modules, Request};
use v_ide_editor::workspace::Workspace;
use v_ide_vault::module::VaultModule;
use v_ide_vault::scan::{scan, scan_with, walk_paths};

fn temp_vault(name: &str) -> PathBuf {
    let root = scratch(&format!("vault-upsert-{name}"));
    fs::create_dir_all(root.join("sub")).unwrap();
    fs::write(root.join("a.md"), "# A\n").unwrap();
    fs::write(root.join("sub/b.md"), "see [[a]] #idea\n").unwrap();
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

fn with_facts(root: &Path) -> (Workspace, Modules) {
    let ws = ws_at(root, None);
    let modules = started(&ws);
    assert_eq!(vault(&modules).index().unwrap().notes().len(), 2);
    (ws, modules)
}

fn announced(requests: &[Request]) -> bool {
    matches!(requests, [request] if is_changed(request))
}

fn backlinks_of(modules: &Modules, path: &str) -> Vec<String> {
    let index = vault(modules).index().unwrap();
    index
        .backlinks(path)
        .iter()
        .map(|n| n.path.clone())
        .collect()
}

#[test]
fn create_with_facts_matches_full_scan() {
    let root = temp_vault("create");
    let (ws, mut modules) = with_facts(&root);
    let abs = root.join("c.md");
    fs::write(&abs, "back to [[a]] #fresh\n").unwrap();
    let requests = modules.event(&ws, &Event::NoteCreate(abs));
    assert!(announced(&requests), "{requests:?}");
    assert_eq!(
        vault(&modules).index().unwrap().notes(),
        scan(&root).unwrap()
    );
    assert_eq!(
        vault(&modules).items().unwrap(),
        walk_paths(&root, false).unwrap()
    );
}

#[test]
fn save_with_facts_replaces_note() {
    let root = temp_vault("save");
    let (ws, mut modules) = with_facts(&root);
    assert_eq!(backlinks_of(&modules, "a.md"), strings(&["sub/b.md"]));
    let abs = root.join("sub/b.md");
    fs::write(&abs, "no links #other\n").unwrap();
    let requests = modules.event(&ws, &Event::NoteSave(abs));
    assert!(announced(&requests), "{requests:?}");
    assert_eq!(
        vault(&modules).index().unwrap().notes(),
        scan(&root).unwrap()
    );
    assert_eq!(backlinks_of(&modules, "a.md"), Vec::<String>::new());
    assert_eq!(vault(&modules).index().unwrap().tags(), strings(&["other"]));
}

#[test]
fn upsert_reads_only_the_event_file() {
    let root = temp_vault("only");
    let (ws, mut modules) = with_facts(&root);
    fs::write(root.join("a.md"), "now [[b]]\n").unwrap();
    let abs = root.join("c.md");
    fs::write(&abs, "# C\n").unwrap();
    let requests = modules.event(&ws, &Event::NoteCreate(abs));
    assert!(announced(&requests), "{requests:?}");
    assert_eq!(backlinks_of(&modules, "sub/b.md"), Vec::<String>::new());
    assert_eq!(
        vault(&modules).items().unwrap(),
        strings(&["a.md", "c.md", "sub/b.md"])
    );
}

#[test]
fn save_of_new_file_with_names_only_adds_path_silently() {
    let root = temp_vault("names");
    let ws = ws_at(&root, None);
    let mut modules = started(&ws);
    assert_eq!(vault(&modules).items().unwrap().len(), 2);
    let abs = root.join("sub/new.md");
    fs::write(&abs, "# New\n").unwrap();
    assert_eq!(modules.event(&ws, &Event::NoteSave(abs)), Vec::new());
    assert_eq!(
        vault(&modules).items().unwrap(),
        strings(&["a.md", "sub/b.md", "sub/new.md"])
    );
}

#[test]
fn paths_outside_the_scan_change_nothing() {
    let root = temp_vault("ignored");
    let (ws, mut modules) = with_facts(&root);
    let outside = scratch("vault-upsert-outside").join("x.md");
    fs::write(&outside, "[[a]]\n").unwrap();
    fs::write(root.join("notes.txt"), "[[a]]\n").unwrap();
    fs::create_dir_all(root.join(".h")).unwrap();
    fs::write(root.join(".h/x.md"), "[[a]]\n").unwrap();
    for abs in [outside, root.join("notes.txt"), root.join(".h/x.md")] {
        let requests = modules.event(&ws, &Event::NoteCreate(abs.clone()));
        assert_eq!(requests, Vec::new(), "{abs:?}");
    }
    assert_eq!(
        vault(&modules).index().unwrap().notes(),
        scan(&root).unwrap()
    );
    assert_eq!(
        vault(&modules).items().unwrap(),
        strings(&["a.md", "sub/b.md"])
    );
}

#[test]
fn hidden_path_counts_when_scan_hidden_is_on() {
    let root = temp_vault("hidden");
    let (ws, mut modules) = with_facts(&root);
    modules.command(&ws, "scan-hidden", "on");
    let abs = root.join(".h/x.md");
    fs::create_dir_all(root.join(".h")).unwrap();
    fs::write(&abs, "[[a]]\n").unwrap();
    let requests = modules.event(&ws, &Event::NoteSave(abs));
    assert!(announced(&requests), "{requests:?}");
    assert_eq!(
        vault(&modules).index().unwrap().notes(),
        scan_with(&root, true).unwrap()
    );
    assert_eq!(
        backlinks_of(&modules, "a.md"),
        strings(&[".h/x.md", "sub/b.md"])
    );
}

#[test]
fn unreadable_saved_note_falls_back_to_names() {
    let root = temp_vault("bad");
    let (ws, mut modules) = with_facts(&root);
    let abs = root.join("sub/b.md");
    fs::write(&abs, [0xff, 0xfe, 0x00]).unwrap();
    assert_eq!(modules.event(&ws, &Event::NoteSave(abs)), Vec::new());
    assert_eq!(
        vault(&modules).items().unwrap(),
        strings(&["a.md", "sub/b.md"])
    );
    let failed = vault(&modules).index().err().unwrap_or_default();
    assert!(failed.starts_with("vault: "), "{failed}");
}
