mod common;

use v_ide_tui::app::App;
use v_ide_view::Depth;

fn app_with_opt(label: &str, script: &str) -> App {
    common::lua_app(common::named_doc("x", "a.md"), label, script)
}

#[test]
fn explicit_depths_are_parsed() {
    for (value, depth) in [
        ("16", Depth::Ansi16),
        ("256", Depth::Ansi256),
        ("truecolor", Depth::TrueColor),
    ] {
        let script = format!("vibook.opt.color_depth = '{value}'");
        let app = app_with_opt(&format!("depth-{value}"), &script);
        assert_eq!(app.color_depth, Some(depth), "{value}");
    }
}

#[test]
fn auto_means_detect() {
    let app = app_with_opt("depth-auto", "vibook.opt.color_depth = 'auto'");
    assert_eq!(app.color_depth, None);
}

#[test]
fn absent_option_means_detect() {
    let app = app_with_opt("depth-absent", "vibook.opt.leader = ','");
    assert_eq!(app.color_depth, None);
}

#[test]
fn unknown_value_is_ignored() {
    let app = app_with_opt("depth-bogus", "vibook.opt.color_depth = 'bogus'");
    assert_eq!(app.color_depth, None);
}
