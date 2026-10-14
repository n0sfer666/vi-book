mod common;

use v_ide_markdown::wrap_mode::WrapMode;
use v_ide_tui::app::App;

fn app_with_opt(label: &str, script: &str) -> App {
    common::lua_app(common::named_doc("x", "a.md"), label, script)
}

#[test]
fn opt_wrap_all_sets_app_wrap() {
    assert_eq!(
        app_with_opt("wrap-all", "vibook.opt.wrap = 'all'").wrap,
        WrapMode::All
    );
}

#[test]
fn opt_wrap_text_sets_app_wrap() {
    assert_eq!(
        app_with_opt("wrap-text", "vibook.opt.wrap = 'text'").wrap,
        WrapMode::Text
    );
}

#[test]
fn opt_wrap_absent_keeps_default_all() {
    assert_eq!(
        app_with_opt("wrap-absent", "vibook.opt.leader = ','").wrap,
        WrapMode::All
    );
}
