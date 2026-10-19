mod common;

use v_ide_markdown::wrap_mode::WrapMode;
use v_ide_tui::app::App;

fn app_with_opt(label: &str, script: &str) -> App {
    common::lua_app(common::named_doc("x", "a.md"), label, script)
}

#[test]
fn opt_wrap_all_sets_module_wrap() {
    let app = app_with_opt("wrap-all", "vibook.opt.wrap = 'all'");
    assert_eq!(common::wrap(&app), Some(WrapMode::All));
}

#[test]
fn opt_wrap_text_sets_module_wrap() {
    let app = app_with_opt("wrap-text", "vibook.opt.wrap = 'text'");
    assert_eq!(common::wrap(&app), Some(WrapMode::Text));
}

#[test]
fn opt_wrap_absent_leaves_module_idle() {
    let app = app_with_opt("wrap-absent", "vibook.opt.leader = ','");
    assert_eq!(common::wrap(&app), None);
}
