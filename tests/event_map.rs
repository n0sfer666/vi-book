use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use v_ide_keys::{Key, Stroke, format_keys};
use v_ide_tui::event_map::to_stroke;

fn notation(code: KeyCode, modifiers: KeyModifiers) -> Option<String> {
    to_stroke(KeyEvent::new(code, modifiers)).map(|stroke| format_keys(&[stroke.key]))
}

fn some(text: &str) -> Option<String> {
    Some(text.to_string())
}

#[test]
fn every_new_key_is_delivered() {
    let none = KeyModifiers::NONE;
    let table = [
        (KeyCode::Tab, "<Tab>"),
        (KeyCode::Delete, "<Del>"),
        (KeyCode::Insert, "<Insert>"),
        (KeyCode::Home, "<Home>"),
        (KeyCode::End, "<End>"),
        (KeyCode::PageUp, "<PageUp>"),
        (KeyCode::PageDown, "<PageDown>"),
        (KeyCode::F(1), "<F1>"),
        (KeyCode::F(12), "<F12>"),
        (KeyCode::Esc, "<Esc>"),
        (KeyCode::Enter, "<CR>"),
        (KeyCode::Backspace, "<BS>"),
        (KeyCode::Left, "<Left>"),
        (KeyCode::Char('x'), "x"),
        (KeyCode::Char(' '), "<Space>"),
    ];
    for (code, expected) in table {
        assert_eq!(notation(code, none), some(expected), "{code:?}");
    }
    assert_eq!(notation(KeyCode::F(13), none), None);
    assert_eq!(notation(KeyCode::CapsLock, none), None);
}

#[test]
fn modifiers_are_kept() {
    let alt = KeyModifiers::ALT;
    let ctrl = KeyModifiers::CONTROL;
    assert_eq!(notation(KeyCode::Char('x'), alt), some("<M-x>"));
    assert_eq!(notation(KeyCode::Char('w'), ctrl), some("<C-W>"));
    assert_eq!(
        notation(KeyCode::Char('x'), KeyModifiers::SUPER),
        some("<D-x>")
    );
    assert_eq!(notation(KeyCode::Up, ctrl | alt), some("<M-C-Up>"));
    assert_eq!(
        notation(KeyCode::Home, KeyModifiers::SHIFT),
        some("<S-Home>")
    );
    assert_eq!(notation(KeyCode::Char('i'), ctrl), some("<Tab>"));
}

#[test]
fn shift_on_characters_is_already_in_the_character() {
    let shift = KeyModifiers::SHIFT;
    assert_eq!(notation(KeyCode::Char('A'), shift), some("A"));
    assert_eq!(notation(KeyCode::Char('!'), shift), some("!"));
    assert_eq!(notation(KeyCode::Char('Ж'), shift), some("Ж"));
    assert_eq!(
        notation(KeyCode::Char('!'), shift | KeyModifiers::ALT),
        some("<M-!>")
    );
    assert_eq!(
        notation(KeyCode::Char('X'), shift | KeyModifiers::ALT),
        some("<M-X>")
    );
    assert_eq!(
        notation(KeyCode::Char('W'), shift | KeyModifiers::CONTROL),
        some("<C-S-W>")
    );
}

#[test]
fn backtab_is_shift_tab() {
    assert_eq!(
        notation(KeyCode::BackTab, KeyModifiers::SHIFT),
        some("<S-Tab>")
    );
    assert_eq!(
        notation(KeyCode::BackTab, KeyModifiers::NONE),
        some("<S-Tab>")
    );
}

#[test]
fn base_layout_slot_is_empty_for_now() {
    let stroke = to_stroke(KeyEvent::new(KeyCode::Char('ц'), KeyModifiers::NONE));
    assert_eq!(
        stroke,
        Some(Stroke {
            key: Key::char('ц'),
            base: None
        })
    );
}
