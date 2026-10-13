mod common;

use v_ide_markdown::wrap_mode::WrapMode;
use v_ide_tui::app::App;
use v_ide_tui::command_line;

fn app() -> App {
    App::new(common::named_doc("x", "a.md"))
}

#[test]
fn wrap_default_is_all() {
    assert_eq!(app().wrap, WrapMode::All);
}

#[test]
fn wrap_command_sets_each_mode() {
    let mut a = app();
    command_line::run(&mut a, "wrap text");
    assert_eq!(a.wrap, WrapMode::Text);
    command_line::run(&mut a, "wrap all");
    assert_eq!(a.wrap, WrapMode::All);
    command_line::run(&mut a, "wrap none");
    assert_eq!(a.wrap, WrapMode::None);
}

#[test]
fn wrap_command_rejects_invalid_mode() {
    let mut a = app();
    a.wrap = WrapMode::All;
    command_line::run(&mut a, "wrap bogus");
    assert_eq!(a.wrap, WrapMode::All, "invalid mode leaves wrap unchanged");
    assert!(a.message.contains("wrap"), "message mentions wrap");
}
