use std::path::PathBuf;

use v_ide_engine::Editor;
use v_ide_tui::app::App;
use v_ide_tui::plugin;

fn config_file(label: &str, script: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("v-ide-config-msg-{}-{label}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("init.lua");
    std::fs::write(&path, script).unwrap();
    path
}

fn load(label: &str, script: &str, warning: Option<&'static str>) -> App {
    let mut app = App::new(Editor::new("x"), "a.md".into());
    plugin::load_config_from(&mut app, Some((config_file(label, script), warning)));
    app
}

#[test]
fn warning_goes_to_status_line() {
    let app = load("warn", "vibook.opt.leader = \" \"", Some("config: old"));
    assert_eq!(app.message, "config: old");
}

#[test]
fn config_error_overrides_warning() {
    let app = load("error", "error(\"boom\")", Some("config: old"));
    assert!(app.message.starts_with("config error:"), "{}", app.message);
}

#[test]
fn no_warning_leaves_status_line_empty() {
    let app = load("clean", "vibook.opt.leader = \" \"", None);
    assert_eq!(app.message, "");
}

#[test]
fn no_config_leaves_status_line_empty() {
    let mut app = App::new(Editor::new("x"), "a.md".into());
    plugin::load_config_from(&mut app, None);
    assert_eq!(app.message, "");
}
