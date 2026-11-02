mod common;

use v_ide_tui::app::App;
use v_ide_tui::lua_bridge;

fn started(label: &str) -> App {
    let mut app = App::new(common::named_doc("x", "a.md"));
    common::open_vault(&mut app, &common::temp_root(label));
    app
}

fn load(label: &str, script: &str, warning: Option<&'static str>) -> App {
    let mut app = started(label);
    let path = common::config_file(label, script);
    lua_bridge::load_config_from(&mut app, Some((path, warning)));
    app
}

#[test]
fn warning_goes_to_status_line() {
    let app = load("warn", "vibook.opt.leader = \" \"", Some("config: old"));
    assert_eq!(app.ui.message, "config: old");
}

#[test]
fn config_error_overrides_warning() {
    let app = load("error", "error(\"boom\")", Some("config: old"));
    assert!(
        app.ui.message.starts_with("config error:"),
        "{}",
        app.ui.message
    );
}

#[test]
fn no_warning_leaves_status_line_empty() {
    let app = load("clean", "vibook.opt.leader = \" \"", None);
    assert_eq!(app.ui.message, "");
}

#[test]
fn no_config_leaves_status_line_empty() {
    let mut app = started("none");
    lua_bridge::load_config_from(&mut app, None);
    assert_eq!(app.ui.message, "");
}

#[test]
fn config_without_startup_activates_lua() {
    let mut app = App::new(common::named_doc("x", "a.md"));
    let path = common::config_file("nostart", "vibook.opt.leader = ','");
    lua_bridge::load_config_from(&mut app, Some((path, Some("config: old"))));
    assert_eq!(app.leader, v_ide_keys::parse_keys(","));
    assert_eq!(app.ui.message, "config: old");
}

#[test]
fn vault_error_on_demand_replaces_warning() {
    let mut app = App::new(common::named_doc("x", "a.md"));
    common::open_vault(&mut app, &common::scratch_path("no-such-vault"));
    let startup = app.ui.message.clone();
    let path = common::config_file("startup-warn", "vibook.opt.leader = ','");
    lua_bridge::load_config_from(&mut app, Some((path, Some("config: old"))));
    let warned = app.ui.message.clone();
    app.open_picker();
    assert_eq!(startup, "");
    assert_eq!(warned, "config: old");
    assert!(app.ui.message.starts_with("vault:"), "{}", app.ui.message);
}

#[test]
fn config_error_overrides_startup_message() {
    let mut app = App::new(common::named_doc("x", "a.md"));
    common::open_vault(&mut app, &common::scratch_path("no-such-vault"));
    let path = common::config_file("startup-error", "error(\"boom\")");
    lua_bridge::load_config_from(&mut app, Some((path, Some("config: old"))));
    assert!(
        app.ui.message.starts_with("config error:"),
        "{}",
        app.ui.message
    );
}
