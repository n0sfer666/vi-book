use vi_book::vim::mode::Mode;
use vi_book::{parse_keys, Editor, Position};

#[test]
fn arrows_parse_as_keys() {
    use vi_book::Key;
    assert_eq!(
        parse_keys("<Up><Down><Left><Right>"),
        vec![Key::Up, Key::Down, Key::Left, Key::Right]
    );
    assert_eq!(parse_keys("<Space>"), vec![Key::Char(' ')]);
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
fn insert_escape_sequence_leaves_mode_without_inserting() {
    let mut ed = Editor::new("");
    ed.set_insert_escape(Some("jj".into()));
    ed.handle_keys("ihellojj");
    assert_eq!(ed.text(), "hello");
    assert_eq!(ed.mode(), Mode::Normal);
}

#[test]
fn insert_escape_does_not_fire_on_single_char() {
    let mut ed = Editor::new("");
    ed.set_insert_escape(Some("jj".into()));
    ed.handle_keys("ijazz");
    assert_eq!(ed.text(), "jazz");
    assert_eq!(ed.mode(), Mode::Insert);
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
