mod common;

use v_ide_tui::app::App;
use v_ide_tui::command_line;
use v_ide_tui::lua_bridge;
use v_ide_vault::module::VaultModule;

const COUNT: &str = "vibook.command('count', function()\n\
                       vibook.buf.set_text(tostring(#vibook.vault.notes()))\n\
                     end)\n";

fn app_with_opt(label: &str, script: &str) -> App {
    let dir = common::temp_root(&format!("opt-hidden-{label}"));
    std::fs::create_dir_all(dir.join(".hidden")).unwrap();
    std::fs::write(dir.join(".hidden/n.md"), "# N\n").unwrap();
    let mut app = App::new(common::named_doc("x", "a.md"));
    common::open_vault(&mut app, &dir);
    common::load_lua(&mut app, &format!("opt-hidden-{label}"), script);
    app
}

fn scan_hidden(app: &App) -> bool {
    app.modules
        .get::<VaultModule>()
        .is_some_and(VaultModule::scan_hidden)
}

#[test]
fn opt_scan_hidden_true_enables_flag() {
    let app = app_with_opt("true", "vibook.opt.scan_hidden = true");
    assert!(scan_hidden(&app));
}

#[test]
fn opt_scan_hidden_absent_stays_off() {
    let app = app_with_opt("absent", "vibook.opt.leader = ','");
    assert!(!scan_hidden(&app));
}

#[test]
fn opt_scan_hidden_is_silent_at_startup() {
    let app = app_with_opt("silent", "vibook.opt.scan_hidden = true");
    assert_eq!(app.ui.message, "");
}

#[test]
fn opt_scan_hidden_keeps_config_warning() {
    let dir = common::temp_root("opt-hidden-warn");
    let mut app = App::new(common::named_doc("x", "a.md"));
    common::open_vault(&mut app, &dir);
    let path = common::config_file("opt-hidden-warn", "vibook.opt.scan_hidden = true");
    lua_bridge::load_config_from(&mut app, Some((path, Some("config: old"))));
    assert_eq!(app.ui.message, "config: old");
}

#[test]
fn opt_scan_hidden_reaches_lua_vault() {
    let mut app = app_with_opt("lua", &format!("{COUNT}vibook.opt.scan_hidden = true"));
    command_line::run(&mut app, "count");
    assert_eq!(app.workspace.current().editor.text(), "1");
}

#[test]
fn opt_scan_hidden_reindex_failure_is_shown() {
    let dir = common::temp_root("opt-hidden-fail");
    let mut app = App::new(common::named_doc("x", "a.md"));
    common::open_vault(&mut app, &dir);
    std::fs::remove_dir_all(&dir).unwrap();
    let path = common::config_file("opt-hidden-fail", "vibook.opt.scan_hidden = true");
    lua_bridge::load_config_from(&mut app, Some((path, Some("config: old"))));
    assert!(
        app.ui.message.starts_with("reindex failed:"),
        "{}",
        app.ui.message
    );
}
