mod common;

use v_ide_keys::{Code, Key, Mods};
use v_ide_tui::pager::{Pager, PagerInput};

fn enter() -> Key {
    Key::new(Code::Enter, Mods::NONE)
}

#[test]
fn small_output_grows_from_bottom_and_accepts_enter_or_replays_a_command() {
    let mut pager = Pager::new("first\nsecond", 40, 8);
    assert_eq!(pager.visible().collect::<Vec<_>>(), vec!["first", "second"]);
    assert_eq!(pager.prompt(), "Press ENTER or type command to continue");
    assert_eq!(pager.key(enter()), PagerInput::Close);
    assert_eq!(
        pager.key(Key::char('j')),
        PagerInput::Replay(Key::char('j'))
    );
    assert_eq!(pager.key(Key::char(':')), PagerInput::Command);
}

#[test]
fn more_pages_scroll_lines_half_screens_screens_and_both_ends() {
    let text = (0..20)
        .map(|n| format!("line {n}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut pager = Pager::new(text, 40, 7);
    assert_eq!(pager.prompt(), "-- More --");
    assert_eq!(pager.key(enter()), PagerInput::Keep);
    assert_eq!(pager.visible().next(), Some("line 1"));
    pager.key(Key::char('d'));
    assert_eq!(pager.visible().next(), Some("line 4"));
    pager.key(Key::char(' '));
    assert_eq!(pager.visible().next(), Some("line 10"));
    pager.key(Key::char('b'));
    assert_eq!(pager.visible().next(), Some("line 4"));
    pager.key(Key::char('g'));
    assert_eq!(pager.visible().next(), Some("line 0"));
    pager.key(Key::char('G'));
    assert_eq!(pager.visible().next(), Some("line 14"));
    assert_eq!(pager.prompt(), "Press ENTER or type command to continue");
    assert_eq!(pager.key(enter()), PagerInput::Close);
}

#[test]
fn messages_wrap_by_display_width_preserving_graphemes_and_escape_control_bytes() {
    let pager = Pager::new("界界e\u{301}X\n\u{1b}[31m", 4, 12);
    assert_eq!(
        pager.visible().collect::<Vec<_>>(),
        vec!["界界", "e\u{301}X", "^[[3", "1m"]
    );
    let pager = Pager::new("👩‍💻x", 2, 5);
    assert_eq!(pager.visible().collect::<Vec<_>>(), vec!["👩‍💻", "x"]);
}

#[test]
fn resize_preserves_the_source_position_and_handles_tiny_sizes() {
    let mut pager = Pager::new("abcdefghij\nklmnopqrst\nuvwxyz", 5, 4);
    pager.key(Key::char('j'));
    assert_eq!(pager.visible().next(), Some("fghij"));
    pager.resize(3, 4);
    assert_eq!(pager.visible().next(), Some("def"));
    pager.resize(0, 0);
    assert_eq!(
        pager.key(Key::new(Code::Esc, Mods::NONE)),
        PagerInput::Close
    );
}
