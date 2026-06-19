use vi_book::markdown::wrap_mode::WrapMode;
use vi_book::ui::app::App;
use vi_book::ui::command_line;
use vi_book::Editor;

fn app() -> App {
    App::new(Editor::new("x"), "a.md".into())
}

#[test]
fn wrap_default_is_none() {
    assert_eq!(app().wrap, WrapMode::None);
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
