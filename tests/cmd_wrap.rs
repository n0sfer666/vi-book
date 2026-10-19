mod common;

use v_ide_markdown::wrap_mode::WrapMode;
use v_ide_tui::app::App;
use v_ide_tui::command_line;

fn app() -> App {
    App::new(common::named_doc("x", "a.md"))
}

#[test]
fn wrap_module_is_idle_until_used() {
    assert_eq!(common::wrap(&app()), None);
}

#[test]
fn wrap_command_sets_each_mode() {
    let mut a = app();
    command_line::run(&mut a, "wrap text");
    assert_eq!(common::wrap(&a), Some(WrapMode::Text));
    command_line::run(&mut a, "wrap all");
    assert_eq!(common::wrap(&a), Some(WrapMode::All));
    command_line::run(&mut a, "wrap none");
    assert_eq!(common::wrap(&a), Some(WrapMode::None));
}

#[test]
fn wrap_command_rejects_invalid_mode() {
    let mut a = app();
    command_line::run(&mut a, "wrap text");
    command_line::run(&mut a, "wrap bogus");
    assert_eq!(
        common::wrap(&a),
        Some(WrapMode::Text),
        "invalid mode leaves wrap unchanged"
    );
    assert!(a.ui.message.contains("wrap"), "message mentions wrap");
}
