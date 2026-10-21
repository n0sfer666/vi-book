mod common;
#[path = "module_host/fakes.rs"]
mod fakes;

use std::fs;
use std::path::PathBuf;

use v_ide_editor::module::Request;
use v_ide_tui::app::App;
use v_ide_tui::requests;

fn temp_root(name: &str) -> PathBuf {
    common::scratch(&format!("module-requests-{name}"))
}

#[test]
fn set_text_replaces_current_buffer() {
    let mut app = App::new(common::named_doc("old", "a.md"));
    requests::apply(&mut app, vec![Request::SetText(String::from("new"))]);
    assert_eq!(app.workspace.current().editor.text(), "new");
    assert!(app.workspace.current().is_dirty());
}

#[test]
fn set_text_with_same_text_keeps_buffer_clean() {
    let mut app = App::new(common::named_doc("same", "a.md"));
    requests::apply(&mut app, vec![Request::SetText(String::from("same"))]);
    assert!(!app.workspace.current().is_dirty());
}

#[test]
fn open_external_resolves_against_cwd() {
    let root = temp_root("external");
    fs::write(root.join("x.rs"), "fn main() {}\n").unwrap();
    let mut app = App::new(common::named_doc("", ""));
    app.workspace.set_cwd(root.clone());
    requests::apply(&mut app, vec![Request::OpenExternal(String::from("x.rs"))]);
    let got = app.external_request.clone();
    let _ = fs::remove_dir_all(&root);
    assert_eq!(got, Some(root.join("x.rs")));
}

#[test]
fn open_external_reports_missing_file() {
    let root = temp_root("missing");
    let mut app = App::new(common::named_doc("", ""));
    app.workspace.set_cwd(root.clone());
    requests::apply(&mut app, vec![Request::OpenExternal(String::from("y.rs"))]);
    let _ = fs::remove_dir_all(&root);
    assert_eq!(app.external_request, None);
    assert_eq!(app.ui.message, "file not found: y.rs");
}

#[test]
fn command_request_runs_command_line() {
    let mut app = App::new(common::named_doc("", ""));
    app.modules = fakes::all();
    requests::apply(&mut app, vec![Request::Command(String::from("echo hi"))]);
    assert_eq!(app.ui.message, "echo hi");
}
