use v_ide_keys::{Code, FKey, Key, Mods, parse_keys};

fn key(code: Code, mods: Mods) -> Key {
    Key::new(code, mods)
}

fn chars(text: &str) -> Vec<Key> {
    text.chars().map(Key::char).collect()
}

fn fkey(n: u8) -> FKey {
    FKey::new(n).unwrap()
}

#[test]
fn plain_characters_are_keys() {
    assert_eq!(parse_keys("dw"), vec![Key::char('d'), Key::char('w')]);
    assert_eq!(parse_keys("ж>"), vec![Key::char('ж'), Key::char('>')]);
}

#[test]
fn special_names_are_case_insensitive() {
    let table = [
        ("<Esc>", Code::Esc),
        ("<CR>", Code::Enter),
        ("<Enter>", Code::Enter),
        ("<return>", Code::Enter),
        ("<BS>", Code::Backspace),
        ("<TAB>", Code::Tab),
        ("<del>", Code::Delete),
        ("<Insert>", Code::Insert),
        ("<Home>", Code::Home),
        ("<End>", Code::End),
        ("<PageUp>", Code::PageUp),
        ("<pagedown>", Code::PageDown),
        ("<Up>", Code::Up),
        ("<Down>", Code::Down),
        ("<Left>", Code::Left),
        ("<Right>", Code::Right),
        ("<F1>", Code::F(fkey(1))),
        ("<f12>", Code::F(fkey(12))),
        ("<Space>", Code::Char(' ')),
        ("<LT>", Code::Char('<')),
        ("<Bar>", Code::Char('|')),
        ("<Bslash>", Code::Char('\\')),
    ];
    for (notation, code) in table {
        assert_eq!(
            parse_keys(notation),
            vec![key(code, Mods::NONE)],
            "{notation}"
        );
    }
}

#[test]
fn modifiers_in_any_order_and_case() {
    let alt_ctrl = Mods {
        ctrl: true,
        alt: true,
        ..Mods::NONE
    };
    assert_eq!(parse_keys("<C-M-x>"), parse_keys("<M-C-x>"));
    assert_eq!(parse_keys("<c-m-x>"), vec![key(Code::Char('x'), alt_ctrl)]);
    assert_eq!(parse_keys("<A-x>"), vec![key(Code::Char('x'), Mods::ALT)]);
    assert_eq!(parse_keys("<a-x>"), parse_keys("<M-x>"));
    assert_eq!(parse_keys("<d-x>"), vec![key(Code::Char('x'), Mods::SUP)]);
    assert_eq!(parse_keys("<s-tab>"), vec![key(Code::Tab, Mods::SHIFT)]);
    assert_eq!(parse_keys("<C-Up>"), vec![key(Code::Up, Mods::CTRL)]);
    let all = Mods {
        ctrl: true,
        alt: true,
        shift: true,
        sup: true,
    };
    assert_eq!(parse_keys("<C-S-M-D-Up>"), vec![key(Code::Up, all)]);
}

#[test]
fn single_character_after_modifiers() {
    assert_eq!(parse_keys("<C-->"), vec![key(Code::Char('-'), Mods::CTRL)]);
    assert_eq!(parse_keys("<M-<>"), vec![key(Code::Char('<'), Mods::ALT)]);
    assert_eq!(parse_keys("<M-<>"), parse_keys("<M-lt>"));
    assert_eq!(parse_keys("<M->>"), vec![key(Code::Char('>'), Mods::ALT)]);
    assert_eq!(parse_keys("<M-C>"), vec![key(Code::Char('C'), Mods::ALT)]);
    assert_eq!(
        parse_keys("<C-\\>"),
        vec![key(Code::Char('\\'), Mods::CTRL)]
    );
    assert_eq!(parse_keys("<M-ж>"), vec![key(Code::Char('ж'), Mods::ALT)]);
}

#[test]
fn unknown_or_unclosed_brackets_are_literal() {
    for text in [
        "<foo>", "<abc", "<S-x-y>", "<C->", "<M-<lt>>", "<<", "< >", "<>", "<F0>", "<F13>", "<x>",
    ] {
        let expected: Vec<Key> = match text {
            "<M-<lt>>" => chars("<M-<>"),
            _ => chars(text),
        };
        assert_eq!(parse_keys(text), expected, "{text}");
    }
    assert_eq!(parse_keys("a<b"), chars("a<b"));
    assert_eq!(parse_keys("<lt>lt>"), chars("<lt>"));
}

#[test]
fn mixed_sequence() {
    assert_eq!(
        parse_keys("i<Tab>x<Esc><C-w>v"),
        vec![
            Key::char('i'),
            key(Code::Tab, Mods::NONE),
            Key::char('x'),
            key(Code::Esc, Mods::NONE),
            Key::ctrl('w'),
            Key::char('v'),
        ]
    );
}
