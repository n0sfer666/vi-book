mod common;

use v_ide_keys::key::Key;
use v_ide_markdown::wrap_mode::WrapMode;
use v_ide_tui::actions;
use v_ide_tui::app::App;

fn app() -> App {
    App::new(common::named_doc("x", "a.md"))
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
    assert_eq!(common::wrap(&a), Some(WrapMode::All));
    assert!(a.ui.menu.is_none(), "menu closes after a leaf choice");
}

#[test]
fn settings_wrap_text_and_none() {
    let mut a = app();
    actions::open_menu(&mut a);
    press(&mut a, 's');
    press(&mut a, 'w');
    press(&mut a, 't');
    assert_eq!(common::wrap(&a), Some(WrapMode::Text));
}

#[test]
fn settings_have_no_scan_hidden_item() {
    let mut a = app();
    actions::open_menu(&mut a);
    press(&mut a, 's');
    let menu = a.ui.menu.as_ref().unwrap();
    assert!(
        menu.items.iter().all(|i| i.action != ":scan-hidden toggle"),
        "scan-hidden is a command only"
    );
}

#[test]
fn esc_steps_back_through_levels_then_closes() {
    let mut a = app();
    actions::open_menu(&mut a);
    press(&mut a, 's');
    press(&mut a, 'w');
    actions::menu_key(&mut a, Some(Key::Esc));
    assert!(
        a.ui.menu.is_some(),
        "esc in wrap submenu returns to settings"
    );
    actions::menu_key(&mut a, Some(Key::Esc));
    assert!(a.ui.menu.is_some(), "esc in settings returns to leader");
    actions::menu_key(&mut a, Some(Key::Esc));
    assert!(a.ui.menu.is_none(), "esc at root closes the menu");
}

#[test]
fn current_wrap_mode_is_marked() {
    let mut a = app();
    common::set_wrap(&mut a, "text");
    actions::open_menu(&mut a);
    press(&mut a, 's');
    press(&mut a, 'w');
    let menu = a.ui.menu.as_ref().unwrap();
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
