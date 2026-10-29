mod common;
#[path = "module_host/fakes.rs"]
mod fakes;

use std::fs;
use std::path::{Path, PathBuf};

use v_ide_editor::module::{Cx, Event, Manifest, Module, Modules, Registration, Request, Trigger};
use v_ide_keys::{Code, Key, Mods};
use v_ide_tui::app::App;
use v_ide_tui::list_panel::ListPanel;
use v_ide_tui::panel::Panel;
use v_ide_tui::picker::Picker;
use v_ide_tui::{command_line, input, requests};

struct Looper;

impl Module for Looper {
    fn activate(&mut self, _cx: &mut Cx) -> Result<(), String> {
        Ok(())
    }

    fn command(&mut self, _cx: &mut Cx, _name: &str, _args: &str) -> Result<(), String> {
        Ok(())
    }

    fn event(&mut self, cx: &mut Cx, ev: &Event) {
        cx.emit(ev.clone());
    }
}

static LOOPER: Manifest = Manifest {
    name: "looper",
    commands: &[],
    triggers: &[Trigger::Event("note_save")],
};

fn temp_vault(name: &str) -> PathBuf {
    let root = common::scratch(&format!("wiring-{name}"));
    fs::write(root.join("a.md"), "# A\n").unwrap();
    fs::write(root.join("b.md"), "# B\n").unwrap();
    root
}

fn app_in(root: &Path, modules: Modules) -> App {
    let mut app = App::new(common::named_doc("", ""));
    app.workspace.set_cwd(root.to_path_buf());
    app.modules = modules;
    app
}

fn app_with(modules: Modules) -> App {
    let mut app = App::new(common::named_doc("", ""));
    app.modules = modules;
    app
}

fn enter() -> Key {
    Key::new(Code::Enter, Mods::NONE)
}

#[test]
fn command_line_runs_module_command() {
    let mut app = app_with(fakes::all());
    command_line::run(&mut app, "echo hi there");
    assert_eq!(app.ui.message, "echo hi there");
}

#[test]
fn unknown_command_still_reports_error() {
    let mut app = app_with(fakes::all());
    command_line::run(&mut app, "nope");
    assert_eq!(app.ui.message, "not an editor command: nope");
}

#[test]
fn failed_activation_is_reported() {
    let mut app = app_with(fakes::all());
    command_line::run(&mut app, "broken");
    assert_eq!(app.ui.message, "boom");
}

#[test]
fn edit_command_opens_note_from_vault_root() {
    let root = temp_vault("edit");
    let mut app = app_in(&root, Modules::new(Vec::new()));
    command_line::run(&mut app, "e b.md");
    let name = app.workspace.current().name.clone();
    let _ = fs::remove_dir_all(&root);
    assert_eq!(name, "b.md");
}

#[test]
fn requests_open_pick_and_list_are_applied() {
    let root = temp_vault("effects");
    let mut app = app_in(&root, fakes::all());
    command_line::run(&mut app, "effects");
    let name = app.workspace.current().name.clone();
    let _ = fs::remove_dir_all(&root);
    assert_eq!(name, "a.md", "Request::Open must open the note");
    let picker = app
        .ui
        .picker
        .as_ref()
        .expect("Request::Pick must open a picker");
    assert_eq!(picker.current().as_deref(), Some("x"));
    assert_eq!(picker.command(), "choose");
    let Some(Panel::List(list)) = app.ui.panel.as_ref() else {
        panic!("Request::List must open a list panel");
    };
    assert_eq!(list.title(), "Title");
    assert_eq!(list.rows(), ["r"]);
    assert_eq!(list.command(), "go");
    assert!(app.ui.panel_focus, "list panel must take focus");
}

#[test]
fn picker_enter_runs_module_command_with_item() {
    let mut app = app_with(fakes::all());
    app.ui.picker = Some(Picker::new(vec![String::from("x")], "echo"));
    input::picker_key(&mut app, enter());
    assert!(app.ui.picker.is_none(), "Enter must close the picker");
    assert_eq!(app.ui.message, "echo x");
}

#[test]
fn picker_enter_falls_back_to_core_command() {
    let root = temp_vault("fallback");
    let mut app = app_in(&root, fakes::all());
    app.ui.picker = Some(Picker::new(vec![String::from("b.md")], "e"));
    input::picker_key(&mut app, enter());
    let name = app.workspace.current().name.clone();
    let _ = fs::remove_dir_all(&root);
    assert_eq!(name, "b.md");
    assert!(app.ui.picker.is_none());
}

#[test]
fn list_panel_enter_runs_module_command_and_closes() {
    let mut app = app_with(fakes::all());
    let list = ListPanel::new("T", vec![String::from("r")], "echo");
    app.ui.panel = Some(Panel::List(list));
    app.ui.panel_focus = true;
    input::panel_key(&mut app, Key::new(Code::Enter, Mods::NONE));
    assert_eq!(app.ui.message, "echo r");
    assert!(app.ui.panel.is_none(), "Enter must close the list panel");
    assert!(!app.ui.panel_focus);
}

#[test]
fn opening_note_emits_note_open_to_modules() {
    let root = temp_vault("open");
    let mut app = app_in(&root, fakes::all());
    app.open_rel("a.md");
    let message = app.ui.message.clone();
    let _ = fs::remove_dir_all(&root);
    assert_eq!(message, "event note_open");
}

#[test]
fn emit_loop_stops_at_depth_limit() {
    let root = temp_vault("loop");
    let modules = Modules::new(vec![Registration {
        manifest: &LOOPER,
        make: || Box::new(Looper),
    }]);
    let mut app = app_in(&root, modules);
    app.open_rel("a.md");
    command_line::run(&mut app, "w");
    let message = app.ui.message.clone();
    let _ = fs::remove_dir_all(&root);
    assert!(
        message.contains("note_save") && message.contains("16"),
        "save must reach modules and the emit loop must stop: {message}"
    );
}

#[test]
fn startup_activates_startup_modules() {
    let mut app = app_with(fakes::all());
    requests::startup(&mut app);
    assert_eq!(app.ui.message, "up");
}

#[test]
fn list_request_with_open_title_toggles_panel() {
    let mut app = app_with(fakes::all());
    let list = |title: &str| Request::List {
        title: title.to_string(),
        rows: vec![String::from("r")],
        command: String::from("go"),
    };
    requests::apply(&mut app, vec![list("One"), list("Two")]);
    assert!(matches!(&app.ui.panel, Some(Panel::List(l)) if l.title() == "Two"));
    requests::apply(&mut app, vec![list("Two")]);
    assert!(app.ui.panel.is_none() && !app.ui.panel_focus);
}
