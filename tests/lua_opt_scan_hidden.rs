mod common;

use v_ide_tui::app::App;
use v_ide_tui::command_line;
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
fn opt_scan_hidden_is_ignored() {
    let app = app_with_opt("ignored", "vibook.opt.scan_hidden = true");
    assert!(!scan_hidden(&app));
    assert_eq!(app.ui.message, "");
}

#[test]
fn opt_scan_hidden_does_not_reach_lua_vault() {
    let mut app = app_with_opt("lua", &format!("{COUNT}vibook.opt.scan_hidden = true"));
    command_line::run(&mut app, "count");
    assert_eq!(app.workspace.current().editor.text(), "0");
}
