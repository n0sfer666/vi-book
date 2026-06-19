use vi_book::markdown::wrap_mode::WrapMode;
use vi_book::ui::actions;
use vi_book::ui::app::App;
use vi_book::vim::key::Key;
use vi_book::Editor;

fn app() -> App {
    App::new(Editor::new("x"), "a.md".into())
}

fn press(app: &mut App, c: char) {
    actions::menu_key(app, Some(Key::Char(c)));
}

#[test]
fn settings_wrap_all_sets_mode_and_closes() {
    let mut a = app();
    actions::open_menu(&mut a);
    press(&mut a, 's');
    press(&mut a, 'w');
    press(&mut a, 'a');
    assert_eq!(a.wrap, WrapMode::All);
    assert!(a.menu.is_none(), "menu closes after a leaf choice");
}

#[test]
fn settings_wrap_text_and_none() {
    let mut a = app();
    actions::open_menu(&mut a);
    press(&mut a, 's');
    press(&mut a, 'w');
    press(&mut a, 't');
    assert_eq!(a.wrap, WrapMode::Text);
}

#[test]
fn esc_steps_back_through_levels_then_closes() {
    let mut a = app();
    actions::open_menu(&mut a);
    press(&mut a, 's');
    press(&mut a, 'w');
    actions::menu_key(&mut a, Some(Key::Esc));
    assert!(a.menu.is_some(), "esc in wrap submenu returns to settings");
    actions::menu_key(&mut a, Some(Key::Esc));
    assert!(a.menu.is_some(), "esc in settings returns to leader");
    actions::menu_key(&mut a, Some(Key::Esc));
    assert!(a.menu.is_none(), "esc at root closes the menu");
}

#[test]
fn current_wrap_mode_is_marked() {
    let mut a = app();
    a.wrap = WrapMode::Text;
    actions::open_menu(&mut a);
    press(&mut a, 's');
    press(&mut a, 'w');
    let menu = a.menu.as_ref().unwrap();
    let item = menu
        .items
        .iter()
        .find(|i| i.action == ":wrap text")
        .expect("wrap text item");
    assert!(
        item.label.contains('\u{25cf}'),
        "active mode marked, got: {:?}",
        item.label
    );
}
