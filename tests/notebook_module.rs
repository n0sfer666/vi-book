use std::fs;
use std::path::{Path, PathBuf};

use v_ide_editor::document::Document;
use v_ide_editor::module::{Event, Modules, Request};
use v_ide_editor::workspace::Workspace;
use v_ide_notebook::daily::{DEFAULT_TEMPLATE, daily_path, expand_template};
use v_ide_notebook::date::Date;
use v_ide_notebook::module::NotebookModule;
use v_ide_vault::module::VaultModule;

fn temp_vault(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "vi-book-notebook-module-{}-{name}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

fn ws_at(root: &Path, current: Option<PathBuf>) -> Workspace {
    let mut doc = Document::new("");
    doc.path = current;
    let mut ws = Workspace::new(doc);
    ws.set_cwd(root.to_path_buf());
    ws
}

fn notebook() -> Modules {
    Modules::new(vec![NotebookModule::registration()])
}

fn run(modules: &mut Modules, ws: &Workspace, name: &str) -> Vec<Request> {
    modules.command(ws, name, "").expect("notebook command")
}

fn created(abs: PathBuf) -> Vec<Request> {
    vec![
        Request::Emit(Event::NoteCreate(abs.clone())),
        Request::Open(abs),
    ]
}

fn on_today(got: impl FnOnce() -> Vec<Request>, expect: impl Fn(Date) -> Vec<Request>) -> Date {
    let before = Date::today();
    let requests = got();
    let after = Date::today();
    [before, after]
        .into_iter()
        .find(|date| requests == expect(*date))
        .unwrap_or_else(|| panic!("unexpected requests: {requests:?}"))
}

#[test]
fn note_create_event_is_named() {
    assert_eq!(
        Event::NoteCreate(PathBuf::from("a.md")).name(),
        "note_create"
    );
}

#[test]
fn notebook_is_inactive_until_command() {
    let root = temp_vault("lazy");
    let ws = ws_at(&root, None);
    let mut modules = notebook();
    assert_eq!(modules.startup(&ws), Vec::new());
    assert!(modules.get::<NotebookModule>().is_none());
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn today_creates_daily_from_template_and_opens_it() {
    let root = temp_vault("today-new");
    let ws = ws_at(&root, None);
    let today = on_today(
        || run(&mut notebook(), &ws, "today"),
        |date| created(root.join(daily_path(date))),
    );
    assert_eq!(
        fs::read_to_string(root.join(daily_path(today))).unwrap(),
        expand_template(DEFAULT_TEMPLATE, today)
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn today_keeps_existing_daily() {
    let root = temp_vault("today-existing");
    let abs = root.join(daily_path(Date::today()));
    fs::create_dir_all(abs.parent().unwrap()).unwrap();
    fs::write(&abs, "mine\n").unwrap();
    let ws = ws_at(&root, None);
    assert_eq!(
        run(&mut notebook(), &ws, "today"),
        vec![Request::Open(abs.clone())]
    );
    assert_eq!(fs::read_to_string(&abs).unwrap(), "mine\n");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn daily_prev_and_next_step_from_current_daily() {
    let root = temp_vault("step");
    let ws = ws_at(&root, Some(root.join("daily/2026-06-12.md")));
    let mut modules = notebook();
    assert_eq!(
        run(&mut modules, &ws, "daily-prev"),
        created(root.join("daily/2026-06-11.md"))
    );
    assert_eq!(
        run(&mut modules, &ws, "daily-next"),
        created(root.join("daily/2026-06-13.md"))
    );
    assert_eq!(
        run(&mut modules, &ws, "daily-prev"),
        vec![Request::Open(root.join("daily/2026-06-11.md"))]
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn daily_prev_outside_daily_steps_from_today() {
    let root = temp_vault("step-today");
    let ws = ws_at(&root, Some(root.join("notes/other.md")));
    on_today(
        || run(&mut notebook(), &ws, "daily-prev"),
        |date| created(root.join(daily_path(date.pred()))),
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn daily_next_for_file_outside_cwd_steps_from_today() {
    let root = temp_vault("outside");
    let elsewhere = temp_vault("outside-other");
    let ws = ws_at(&root, Some(elsewhere.join("daily/2026-06-12.md")));
    on_today(
        || run(&mut notebook(), &ws, "daily-next"),
        |date| created(root.join(daily_path(date.succ()))),
    );
    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&elsewhere);
}

fn assert_daily_failed(requests: &[Request]) {
    assert_eq!(requests.len(), 1, "{requests:?}");
    assert!(matches!(&requests[0], Request::Message(m) if m.starts_with("daily failed: ")));
}

#[test]
fn daily_failure_reports_message() {
    let root = temp_vault("fail");
    fs::write(root.join("daily"), "not a dir").unwrap();
    let ws = ws_at(&root, None);
    assert_daily_failed(&run(&mut notebook(), &ws, "today"));
    let _ = fs::remove_dir_all(&root);
}

#[cfg(unix)]
#[test]
fn daily_write_failure_reports_message() {
    use std::os::unix::fs::PermissionsExt;
    let root = temp_vault("fail-write");
    let dir = root.join("daily");
    fs::create_dir_all(&dir).unwrap();
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o555)).unwrap();
    let ws = ws_at(&root, None);
    let requests = run(&mut notebook(), &ws, "today");
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).unwrap();
    assert_daily_failed(&requests);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn daily_commands_reject_arguments() {
    let root = temp_vault("args");
    let ws = ws_at(&root, None);
    let mut modules = notebook();
    for name in ["today", "daily-prev", "daily-next"] {
        assert_eq!(
            modules.command(&ws, name, " foo "),
            Some(vec![Request::Message(format!(
                "{name}: takes no arguments"
            ))])
        );
    }
    assert!(!root.join("daily").exists());
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn vault_sees_daily_after_note_create_event() {
    let root = temp_vault("with-vault");
    let ws = ws_at(&root, None);
    let mut modules = Modules::new(vec![
        VaultModule::registration(),
        NotebookModule::registration(),
    ]);
    assert!(matches!(
        modules.startup(&ws).as_slice(),
        [Request::Emit(Event::VaultChanged { .. })]
    ));
    let today = on_today(
        || run(&mut modules, &ws, "today"),
        |date| created(root.join(daily_path(date))),
    );
    assert!(matches!(
        modules
            .event(&ws, &Event::NoteCreate(root.join(daily_path(today))))
            .as_slice(),
        [Request::Emit(Event::VaultChanged { .. })]
    ));
    let items = modules
        .get::<VaultModule>()
        .map(VaultModule::items)
        .unwrap_or_default();
    assert_eq!(items, vec![daily_path(today)]);
    let _ = fs::remove_dir_all(&root);
}
