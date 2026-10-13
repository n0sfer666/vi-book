mod common;

use v_ide_lua::host::PluginHost;
use v_ide_markdown::wrap_mode::WrapMode;
use v_ide_tui::app::App;
use v_ide_tui::plugin;

fn app_with_opt(script: &str) -> App {
    let mut app = App::new(common::named_doc("x", "a.md"));
    let host = PluginHost::new().unwrap();
    host.load(script).unwrap();
    app.host = Some(host);
    plugin::apply_opts(&mut app);
    app
}

#[test]
fn opt_wrap_all_sets_app_wrap() {
    assert_eq!(app_with_opt("vibook.opt.wrap = 'all'").wrap, WrapMode::All);
}

#[test]
fn opt_wrap_text_sets_app_wrap() {
    assert_eq!(
        app_with_opt("vibook.opt.wrap = 'text'").wrap,
        WrapMode::Text
    );
}

#[test]
fn opt_wrap_absent_keeps_default_all() {
    assert_eq!(app_with_opt("vibook.opt.leader = ','").wrap, WrapMode::All);
}
