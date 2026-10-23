use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use v_ide_editor::module::{Modules, Request};
use v_ide_editor::workspace::Workspace;
use v_ide_vault::module::VaultModule;
use v_ide_vault::scan::{scan, walk_paths};

use super::scratch;
use super::vault::{is_changed, items, ws_at};

const DEADLINE: Duration = Duration::from_secs(5);
const STEP: Duration = Duration::from_millis(50);
const QUIET: Duration = Duration::from_millis(700);
const PROBE: &str = "probe.md";
const SENTINEL: &str = "sentinel.md";

pub fn temp_vault(name: &str) -> PathBuf {
    let root = scratch(&format!("vault-watch-{name}"));
    fs::create_dir_all(root.join("sub")).unwrap();
    fs::write(root.join("a.md"), "# A\n").unwrap();
    fs::write(root.join("sub/b.md"), "see [[a]] #idea\n").unwrap();
    root
}

pub fn vault(modules: &Modules) -> &VaultModule {
    modules.get::<VaultModule>().unwrap()
}

pub fn started(ws: &Workspace) -> Modules {
    let mut modules = Modules::new(vec![VaultModule::registration()]);
    modules.startup(ws);
    modules
}

pub fn with_facts(root: &Path) -> (Workspace, Modules) {
    let ws = ws_at(root, None);
    let mut modules = started(&ws);
    let notes = vault(&modules).index().unwrap().notes().len();
    ready(root, &ws, &mut modules);
    assert_eq!(vault(&modules).index().unwrap().notes().len(), notes);
    (ws, modules)
}

pub fn with_names(root: &Path) -> (Workspace, Modules) {
    let ws = ws_at(root, None);
    let mut modules = started(&ws);
    let names = items(&modules);
    ready(root, &ws, &mut modules);
    assert_eq!(items(&modules), names);
    (ws, modules)
}

fn has(modules: &Modules, rel: &str) -> bool {
    items(modules).iter().any(|item| item == rel)
}

fn ready(root: &Path, ws: &Workspace, modules: &mut Modules) {
    let probe = root.join(PROBE);
    let start = Instant::now();
    while !has(modules, PROBE) {
        assert!(start.elapsed() < DEADLINE, "watcher never became ready");
        fs::write(&probe, format!("{:?}\n", start.elapsed())).unwrap();
        modules.tick(ws);
        thread::sleep(STEP);
    }
    fs::remove_file(&probe).unwrap();
    settle(ws, modules, |modules| !has(modules, PROBE));
}

pub fn settle(
    ws: &Workspace,
    modules: &mut Modules,
    done: impl Fn(&Modules) -> bool,
) -> Vec<Vec<Request>> {
    let start = Instant::now();
    let mut ticks = Vec::new();
    while start.elapsed() < DEADLINE {
        ticks.push(modules.tick(ws));
        if done(modules) {
            return ticks;
        }
        thread::sleep(STEP);
    }
    panic!("vault did not settle: {ticks:?}");
}

pub fn message(ws: &Workspace, modules: &mut Modules, prefix: &str) -> String {
    let start = Instant::now();
    while start.elapsed() < DEADLINE {
        for request in modules.tick(ws) {
            if let Request::Message(text) = request
                && text.starts_with(prefix)
            {
                return text;
            }
        }
        thread::sleep(STEP);
    }
    panic!("no message starting with {prefix:?}");
}

pub fn quiet(ws: &Workspace, modules: &mut Modules) -> Vec<Request> {
    let start = Instant::now();
    let mut seen = Vec::new();
    while start.elapsed() < QUIET {
        seen.extend(modules.tick(ws));
        thread::sleep(STEP);
    }
    seen
}

pub fn alive(root: &Path, ws: &Workspace, modules: &mut Modules) -> Vec<Vec<Request>> {
    fs::write(root.join(SENTINEL), "# Sentinel\n").unwrap();
    settle(ws, modules, |modules| has(modules, SENTINEL))
}

pub fn facts_match(root: &Path) -> impl Fn(&Modules) -> bool {
    let root = root.to_path_buf();
    move |modules| {
        vault(modules)
            .index()
            .is_ok_and(|index| index.notes() == scan(&root).unwrap())
    }
}

pub fn names_match(root: &Path) -> impl Fn(&Modules) -> bool {
    let root = root.to_path_buf();
    move |modules| vault(modules).items().ok() == walk_paths(&root, false).ok()
}

pub fn announced(ticks: &[Vec<Request>]) -> bool {
    announcements(ticks) > 0
}

pub fn announcements(ticks: &[Vec<Request>]) -> usize {
    ticks.iter().flatten().filter(|r| is_changed(r)).count()
}
