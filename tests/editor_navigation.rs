use v_ide_engine::Editor;
use v_ide_engine::mode::Mode;
use v_ide_keys::parse_keys;
use v_ide_text::Position;

#[test]
fn arrows_parse_as_keys() {
    use v_ide_keys::{Code, Key, Mods};
    assert_eq!(
        parse_keys("<Up><Down><Left><Right>"),
        [Code::Up, Code::Down, Code::Left, Code::Right]
            .map(|code| Key::new(code, Mods::NONE))
            .to_vec()
    );
    assert_eq!(parse_keys("<Space>"), vec![Key::char(' ')]);
}

#[test]
fn arrow_keys_move_like_hjkl() {
    let mut ed = Editor::new("abc\ndef");
    ed.set_cursor(Position::new(0, 0));
    ed.handle_keys("<Down><Right><Right>");
    assert_eq!(ed.cursor(), Position::new(1, 2));
    ed.handle_keys("<Up><Left>");
    assert_eq!(ed.cursor(), Position::new(0, 1));
}

#[test]
fn insert_escape_disabled_by_default_in_library() {
    let mut ed = Editor::new("");
    ed.handle_keys("ijj");
    assert_eq!(ed.text(), "jj");
    assert_eq!(ed.mode(), Mode::Insert);
}

#[test]
fn arrow_moves_cursor_in_insert_mode() {
    let mut ed = Editor::new("abcd");
    ed.set_cursor(Position::new(0, 2));
    ed.handle_keys("i<Left>X");
    assert_eq!(ed.text(), "aXbcd");
}
