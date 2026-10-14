mod common;

use v_ide_lua::host::PluginHost;
use v_ide_tui::app::App;
use v_ide_tui::plugin;
use v_ide_vault::module::VaultModule;

fn app_with_opt(label: &str, script: &str) -> App {
    let dir =
        std::env::temp_dir().join(format!("vi-book-opt-hidden-{}-{label}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut app = App::new(common::named_doc("x", "a.md"));
    common::open_vault(&mut app, &dir);
    let host = PluginHost::new().unwrap();
    host.load(script).unwrap();
    app.host = Some(host);
    plugin::apply_opts(&mut app);
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
