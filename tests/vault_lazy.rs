use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use v_ide_editor::document::Document;
use v_ide_editor::module::{Cx, Event, Manifest, Module, Modules, Registration, Request};
use v_ide_editor::workspace::Workspace;
use v_ide_vault::module::VaultModule;

static PROBE: Manifest = Manifest {
    name: "probe",
    commands: &["probe"],
    triggers: &[],
};

struct Probe;

impl Module for Probe {
    fn activate(&mut self, _cx: &mut Cx<'_>) -> Result<(), String> {
        Ok(())
    }

    fn command(&mut self, cx: &mut Cx<'_>, _name: &str, args: &str) -> Result<(), String> {
        let vault = cx.vault().ok_or("no vault")?;
        cx.message(format!(
            "{}|{}|{}",
            vault.notes()?.join(","),
            vault.tags()?.join(","),
            vault.backlinks(args)?.join(",")
        ));
        Ok(())
    }

    fn event(&mut self, _cx: &mut Cx<'_>, _ev: &Event) {}
}

fn temp_vault(name: &str) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("vi-book-vault-lazy-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    root
}

fn seed(root: &Path) {
    fs::create_dir_all(root.join("sub")).unwrap();
    fs::write(root.join("a.md"), "# A\n").unwrap();
    fs::write(root.join("sub/b.md"), "see [[a]]\n").unwrap();
    fs::write(root.join("t.md"), "text #idea [[a]] [[a]] [[t]]\n").unwrap();
}

fn ws_at(root: &Path) -> Workspace {
    let mut ws = Workspace::new(Document::new(""));
    ws.set_cwd(root.to_path_buf());
    ws
}

fn modules() -> Modules {
    Modules::new(vec![
        VaultModule::registration(),
        Registration {
            manifest: &PROBE,
            make: || Box::new(Probe),
        },
    ])
}

fn items(modules: &Modules) -> Vec<String> {
    modules
        .get::<VaultModule>()
        .and_then(|vault| vault.items().ok())
        .unwrap_or_default()
}

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

#[test]
fn startup_scans_nothing_until_needed() {
    let root = temp_vault("startup");
    seed(&root);
    let ws = ws_at(&root);
    let mut modules = modules();
    assert_eq!(modules.startup(&ws), Vec::new());
    fs::write(root.join("late.md"), "# L\n").unwrap();
    let first = items(&modules);
    fs::write(root.join("later.md"), "# L\n").unwrap();
    let second = items(&modules);
    let _ = fs::remove_dir_all(&root);
    assert_eq!(first, strings(&["a.md", "late.md", "sub/b.md", "t.md"]));
    assert_eq!(second, first);
}

#[test]
fn missing_cwd_reports_on_demand_and_retries() {
    let root = temp_vault("missing");
    let ws = ws_at(&root);
    let mut modules = modules();
    assert_eq!(modules.startup(&ws), Vec::new());
    assert!(modules.get::<VaultModule>().is_some());
    let failed = modules.command(&ws, "pick", "").unwrap();
    assert!(
        matches!(failed.as_slice(), [Request::Message(m)] if m.starts_with("vault: ")),
        "{failed:?}"
    );
    assert!(items(&modules).is_empty());
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("x.md"), "# X\n").unwrap();
    let picked = modules.command(&ws, "pick", "");
    let _ = fs::remove_dir_all(&root);
    let expected = Request::Pick {
        items: strings(&["x.md"]),
        command: String::from("e"),
    };
    assert_eq!(picked, Some(vec![expected]));
}

#[test]
fn query_serves_snapshot_on_first_call() {
    let root = temp_vault("query");
    seed(&root);
    let ws = ws_at(&root);
    let mut modules = modules();
    modules.startup(&ws);
    let linked = modules.command(&ws, "probe", "a.md");
    let selfish = modules.command(&ws, "probe", "t.md");
    let _ = fs::remove_dir_all(&root);
    let notes = "a.md,sub/b.md,t.md|idea";
    let msg = |text: String| Some(vec![Request::Message(text)]);
    assert_eq!(linked, msg(format!("{notes}|sub/b.md,t.md")));
    assert_eq!(selfish, msg(format!("{notes}|")));
}

#[test]
fn note_create_before_scan_is_silent() {
    let root = temp_vault("note-create");
    seed(&root);
    let ws = ws_at(&root);
    let mut modules = modules();
    modules.startup(&ws);
    let abs = root.join("fresh.md");
    fs::write(&abs, "# F\n").unwrap();
    let requests = modules.event(&ws, &Event::NoteCreate(abs));
    let after = items(&modules);
    let _ = fs::remove_dir_all(&root);
    assert_eq!(requests, Vec::new());
    assert_eq!(after, strings(&["a.md", "fresh.md", "sub/b.md", "t.md"]));
}

#[test]
fn scan_hidden_before_scan_only_sets_flag() {
    let root = temp_vault("hidden");
    seed(&root);
    let ws = ws_at(&root);
    let mut modules = modules();
    modules.startup(&ws);
    let on = modules.command(&ws, "scan-hidden", "on");
    fs::create_dir_all(root.join(".h")).unwrap();
    fs::write(root.join(".h/x.md"), "# X\n").unwrap();
    let after = items(&modules);
    let _ = fs::remove_dir_all(&root);
    let message = Request::Message(String::from("scan hidden folders: on"));
    assert_eq!(on, Some(vec![message]));
    assert_eq!(after, strings(&[".h/x.md", "a.md", "sub/b.md", "t.md"]));
}

#[test]
fn note_create_after_scan_announces_snapshot() {
    let root = temp_vault("snapshot");
    seed(&root);
    let ws = ws_at(&root);
    let mut modules = modules();
    modules.startup(&ws);
    let vault = modules.get::<VaultModule>().unwrap();
    assert_eq!(vault.index().unwrap().notes().len(), 3);
    let abs = root.join("c.md");
    fs::write(&abs, "back to [[a]]\n").unwrap();
    let requests = modules.event(&ws, &Event::NoteCreate(abs));
    let _ = fs::remove_dir_all(&root);
    let expected = Event::VaultChanged {
        notes: strings(&["a.md", "c.md", "sub/b.md", "t.md"]),
        tags: strings(&["idea"]),
        backlinks: HashMap::from([(String::from("a.md"), strings(&["c.md", "sub/b.md", "t.md"]))]),
    };
    assert_eq!(requests, vec![Request::Emit(expected)]);
}
