use v_ide_keys::{Code, Key, Mods};
use v_ide_tui::pager::{Pager, PagerInput};

fn enter() -> Key {
    Key::new(Code::Enter, Mods::NONE)
}

#[test]
fn more_reaches_hit_enter_only_after_requesting_past_the_last_line() {
    let mut pager = Pager::new("0\n1\n2\n3\n4\n5\n6", 40, 4);
    pager.key(Key::char(' '));
    assert_eq!(pager.visible().next(), Some("3"));
    pager.key(enter());
    assert_eq!(pager.visible().next(), Some("4"));
    assert_eq!(pager.prompt(), "-- More --");
    pager.key(enter());
    assert_eq!(pager.prompt(), "Press ENTER or type command to continue");
    assert_eq!(pager.key(Key::char('j')), PagerInput::Keep);
    assert_eq!(pager.key(enter()), PagerInput::Close);
}

#[test]
fn half_page_uses_terminal_height_and_control_does_not_alias_the_letter() {
    let text = (0..30)
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join("\n");
    let mut pager = Pager::new(text, 40, 8);
    pager.key(Key::char('d'));
    assert_eq!(pager.visible().next(), Some("4"));
    pager.key(Key::ctrl('d'));
    assert_eq!(pager.visible().next(), Some("4"));
    pager.key(Key::char('u'));
    assert_eq!(pager.visible().next(), Some("0"));
}

#[test]
fn hit_enter_consumes_space_and_scroll_back_keys_but_replays_escape() {
    let mut pager = Pager::new("one\ntwo", 40, 8);
    for key in [
        Key::char('k'),
        Key::char('b'),
        Key::char('u'),
        Key::char('g'),
    ] {
        assert_eq!(pager.key(key), PagerInput::Keep);
    }
    assert_eq!(pager.key(Key::char(' ')), PagerInput::Close);
    assert_eq!(pager.key(Key::ctrl('j')), PagerInput::Close);
    let escape = Key::new(Code::Esc, Mods::NONE);
    assert_eq!(pager.key(escape), PagerInput::Replay(escape));
}
