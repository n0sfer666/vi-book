use v_ide_keys::{Code, FKey, Key, Mods, Stroke, format_keys, parse_keys};

fn one(notation: &str) -> Key {
    let keys = parse_keys(notation);
    assert_eq!(keys.len(), 1, "{notation}");
    keys[0]
}

fn with(ctrl: bool, alt: bool, shift: bool) -> Mods {
    Mods {
        ctrl,
        alt,
        shift,
        sup: false,
    }
}

#[test]
fn getters_expose_code_and_mods() {
    let key = Key::new(Code::Home, Mods::SHIFT);
    assert_eq!(key.code(), Code::Home);
    assert_eq!(key.mods(), Mods::SHIFT);
    assert_eq!(Key::char('x').mods(), Mods::NONE);
    assert_eq!(Key::ctrl('w'), Key::new(Code::Char('w'), Mods::CTRL));
}

#[test]
fn shift_on_ascii_letter_without_ctrl_becomes_case() {
    assert_eq!(Key::new(Code::Char('a'), Mods::SHIFT), Key::char('A'));
    assert_eq!(Key::new(Code::Char('A'), Mods::SHIFT), Key::char('A'));
    assert_eq!(one("<S-a>"), Key::char('A'));
    assert_eq!(one("<M-S-x>"), Key::new(Code::Char('X'), Mods::ALT));
    assert_eq!(one("<D-S-x>"), Key::new(Code::Char('X'), Mods::SUP));
}

#[test]
fn shift_is_kept_elsewhere() {
    assert_eq!(one("<S-!>").mods(), Mods::SHIFT);
    assert_eq!(one("<S-ж>"), Key::new(Code::Char('ж'), Mods::SHIFT));
    assert_eq!(one("<M-S-ж>").mods(), with(false, true, true));
    assert_eq!(one("<S-Tab>"), Key::new(Code::Tab, Mods::SHIFT));
    assert_eq!(one("<S-Space>"), Key::new(Code::Char(' '), Mods::SHIFT));
}

#[test]
fn ctrl_letter_is_lowercase_and_keeps_shift() {
    assert_eq!(Key::ctrl('W'), Key::ctrl('w'));
    assert_eq!(one("<C-W>"), Key::ctrl('w'));
    assert_eq!(
        one("<C-S-w>"),
        Key::new(Code::Char('w'), with(true, false, true))
    );
    assert_eq!(one("<C-S-W>"), one("<C-S-w>"));
    assert_eq!(one("<C-Ж>").code(), Code::Char('Ж'));
    assert_eq!(one("<C-h>"), Key::ctrl('h'));
}

#[test]
fn ctrl_aliases_fold_like_nvim() {
    assert_eq!(one("<C-[>"), Key::new(Code::Esc, Mods::NONE));
    assert_eq!(Key::ctrl('i'), Key::new(Code::Tab, Mods::NONE));
    assert_eq!(one("<C-I>"), Key::new(Code::Tab, Mods::NONE));
    assert_eq!(one("<C-m>"), Key::new(Code::Enter, Mods::NONE));
    assert_eq!(one("<C-S-i>"), Key::new(Code::Tab, Mods::SHIFT));
    assert_eq!(one("<C-M-m>"), Key::new(Code::Enter, Mods::ALT));
    assert_eq!(one("<M-C-[>"), Key::new(Code::Esc, Mods::ALT));
    assert_eq!(one("<C-D-i>"), Key::new(Code::Tab, Mods::SUP));
}

#[test]
fn plain_char_only_without_modifiers() {
    assert_eq!(Key::char('x').plain_char(), Some('x'));
    assert_eq!(one("<S-a>").plain_char(), Some('A'));
    assert_eq!(one("<Space>").plain_char(), Some(' '));
    assert_eq!(one("<M-x>").plain_char(), None);
    assert_eq!(Key::ctrl('w').plain_char(), None);
    assert_eq!(one("<S-!>").plain_char(), None);
    assert_eq!(one("<Esc>").plain_char(), None);
}

#[test]
fn stroke_carries_key_and_base_slot() {
    let stroke = Stroke {
        key: Key::char('ц'),
        base: Some(Key::char('w')),
    };
    assert_eq!(stroke.key, Key::char('ц'));
    assert_eq!(stroke.base, Some(Key::char('w')));
}

#[test]
fn control_characters_are_ctrl_keys_like_nvim() {
    assert_eq!(Key::char('\t'), Key::new(Code::Tab, Mods::NONE));
    assert_eq!(Key::char('\r'), Key::new(Code::Enter, Mods::NONE));
    assert_eq!(Key::char('\x1b'), Key::new(Code::Esc, Mods::NONE));
    assert_eq!(Key::char('\x01'), Key::ctrl('a'));
    assert_eq!(Key::char('\x08'), Key::ctrl('h'));
    assert_eq!(Key::char('\n'), Key::ctrl('j'));
    assert_eq!(Key::char('\x1a'), Key::ctrl('z'));
    assert_eq!(Key::char('\x00'), Key::ctrl('@'));
    assert_eq!(Key::char('\x1c'), Key::ctrl('\\'));
    assert_eq!(Key::char('\x1f'), Key::ctrl('_'));
    assert_eq!(
        Key::new(Code::Char('\x1b'), Mods::ALT),
        Key::new(Code::Esc, Mods::ALT)
    );
    assert_eq!(Key::char('\x7f').plain_char(), Some('\x7f'));
    assert_eq!(
        format_keys(&parse_keys("i\tx\r\x1b\x01")),
        "i<Tab>x<CR><Esc><C-A>"
    );
}

#[test]
fn function_keys_are_f1_to_f12() {
    assert_eq!(FKey::new(0), None);
    assert_eq!(FKey::new(13), None);
    assert_eq!(FKey::new(1).map(FKey::n), Some(1));
    assert_eq!(FKey::new(12).map(FKey::n), Some(12));
    let f5 = FKey::new(5).map(|f| Key::new(Code::F(f), Mods::NONE));
    assert_eq!(f5, parse_keys("<F5>").first().copied());
}
