use v_ide_keys::{Code, FKey, Key, Mods, format_keys, parse_keys};

const CANONICAL: [(&str, &str); 72] = [
    ("<C-w>", "<C-W>"),
    ("<c-w>", "<C-W>"),
    ("<C-S-w>", "<C-S-W>"),
    ("<M-x>", "<M-x>"),
    ("<A-x>", "<M-x>"),
    ("<M-X>", "<M-X>"),
    ("<M-S-x>", "<M-X>"),
    ("<C-M-x>", "<M-C-X>"),
    ("<S-a>", "A"),
    ("<S-Tab>", "<S-Tab>"),
    ("<s-tab>", "<S-Tab>"),
    ("<C-Up>", "<C-Up>"),
    ("<S-Up>", "<S-Up>"),
    ("<D-x>", "<D-x>"),
    ("<D-S-x>", "<D-X>"),
    ("<D-C-x>", "<C-D-X>"),
    ("<D-M-x>", "<M-D-x>"),
    ("<C-S-M-D-Up>", "<M-C-S-D-Up>"),
    ("<S-C-Up>", "<C-S-Up>"),
    ("<D-S-ж>", "<S-D-ж>"),
    ("<del>", "<Del>"),
    ("<Insert>", "<Insert>"),
    ("<Home>", "<Home>"),
    ("<End>", "<End>"),
    ("<PageUp>", "<PageUp>"),
    ("<PageDown>", "<PageDown>"),
    ("<F1>", "<F1>"),
    ("<F12>", "<F12>"),
    ("<S-F5>", "<S-F5>"),
    ("<C-F12>", "<C-F12>"),
    ("<Space>", "<Space>"),
    ("<LT>", "<lt>"),
    ("<Bar>", "|"),
    ("<Bslash>", "\\"),
    ("<Enter>", "<CR>"),
    ("<Return>", "<CR>"),
    ("<Esc>", "<Esc>"),
    ("<BS>", "<BS>"),
    ("<C-Space>", "<C-Space>"),
    ("<M-Space>", "<M-Space>"),
    ("<S-Space>", "<S-Space>"),
    ("<S-CR>", "<S-CR>"),
    ("<M-CR>", "<M-CR>"),
    ("<S-!>", "<S-!>"),
    ("<C-!>", "<C-!>"),
    ("<C-1>", "<C-1>"),
    ("<M-S-1>", "<M-S-1>"),
    ("<S-ж>", "<S-ж>"),
    ("<M-S-ж>", "<M-S-ж>"),
    ("<C-ж>", "<C-ж>"),
    ("<M-Ж>", "<M-Ж>"),
    ("<C-[>", "<Esc>"),
    ("<C-i>", "<Tab>"),
    ("<C-m>", "<CR>"),
    ("<C-h>", "<C-H>"),
    ("<C-S-i>", "<S-Tab>"),
    ("<C-M-m>", "<M-CR>"),
    ("<M-C-[>", "<M-Esc>"),
    ("<M-lt>", "<M-lt>"),
    ("<M-<>", "<M-lt>"),
    ("<S-<>", "<S-lt>"),
    ("<M-Bar>", "<M-Bar>"),
    ("<S-Bar>", "<S-Bar>"),
    ("<M-Bslash>", "<M-Bslash>"),
    ("<C-Bslash>", "<C-\\>"),
    ("<C-->", "<C-->"),
    ("<C-{>", "<C-{>"),
    ("<C-Tab>", "<C-Tab>"),
    ("<foo>", "<lt>foo>"),
    ("<S-x-y>", "<lt>S-x-y>"),
    ("<M-<lt>>", "<lt>M-<lt>>"),
    ("< >", "<lt><Space>>"),
];

fn fkey(n: u8) -> FKey {
    FKey::new(n).unwrap()
}

#[test]
fn format_is_canonical_like_keytrans() {
    for (notation, canonical) in CANONICAL {
        assert_eq!(format_keys(&parse_keys(notation)), canonical, "{notation}");
    }
    assert_eq!(format_keys(&parse_keys("a<b<<")), "a<lt>b<lt><lt>");
    assert_eq!(format_keys(&parse_keys("<abc")), "<lt>abc");
}

#[test]
fn display_matches_format() {
    let key = Key::new(Code::Char('w'), Mods::CTRL);
    assert_eq!(key.to_string(), "<C-W>");
    assert_eq!(Key::char('<').to_string(), "<lt>");
    assert_eq!(Key::new(Code::F(fkey(5)), Mods::NONE).to_string(), "<F5>");
}

#[test]
fn canonical_form_is_a_fixed_point() {
    for (_, canonical) in CANONICAL {
        assert_eq!(
            format_keys(&parse_keys(canonical)),
            canonical,
            "{canonical}"
        );
    }
}

#[test]
fn parse_of_format_returns_the_keys() {
    let codes = [
        Code::Esc,
        Code::Enter,
        Code::Tab,
        Code::Backspace,
        Code::Delete,
        Code::Insert,
        Code::Home,
        Code::End,
        Code::PageUp,
        Code::PageDown,
        Code::Up,
        Code::Down,
        Code::Left,
        Code::Right,
        Code::F(fkey(1)),
        Code::F(fkey(12)),
    ];
    let chars = "aAwWiImM[!1<> |\\-CDSжЖ";
    let all = codes
        .into_iter()
        .chain(chars.chars().map(Code::Char))
        .collect::<Vec<Code>>();
    for bits in 0..16u8 {
        let mods = Mods {
            ctrl: bits & 1 != 0,
            alt: bits & 2 != 0,
            shift: bits & 4 != 0,
            sup: bits & 8 != 0,
        };
        let keys: Vec<Key> = all.iter().map(|&code| Key::new(code, mods)).collect();
        for key in &keys {
            assert_eq!(parse_keys(&key.to_string()), vec![*key], "{key}");
        }
        assert_eq!(parse_keys(&format_keys(&keys)), keys);
    }
}
