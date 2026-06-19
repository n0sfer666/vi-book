use vi_book::lua::host::PluginHost;
use vi_book::markdown::wrap_mode::WrapMode;
use vi_book::ui::app::App;
use vi_book::ui::plugin;
use vi_book::Editor;

fn app_with_opt(script: &str) -> App {
    let mut app = App::new(Editor::new("x"), "a.md".into());
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
fn opt_wrap_absent_keeps_none() {
    assert_eq!(app_with_opt("vibook.opt.leader = ','").wrap, WrapMode::None);
}
