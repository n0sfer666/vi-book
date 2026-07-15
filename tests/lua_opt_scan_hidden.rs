use vi_book::lua::host::PluginHost;
use vi_book::ui::app::App;
use vi_book::ui::plugin;
use vi_book::Editor;

fn app_with_opt(label: &str, script: &str) -> App {
    let dir =
        std::env::temp_dir().join(format!("vi-book-opt-hidden-{}-{label}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut app = App::new(Editor::new("x"), "a.md".into());
    app.session.root = dir;
    let host = PluginHost::new().unwrap();
    host.load(script).unwrap();
    app.host = Some(host);
    plugin::apply_opts(&mut app);
    app
}

#[test]
fn opt_scan_hidden_true_enables_flag() {
    let app = app_with_opt("true", "vibook.opt.scan_hidden = true");
    assert!(app.session.scan_hidden);
}

#[test]
fn opt_scan_hidden_absent_stays_off() {
    let app = app_with_opt("absent", "vibook.opt.leader = ','");
    assert!(!app.session.scan_hidden);
}
