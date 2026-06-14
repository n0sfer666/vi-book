use vi_book::vim::mode::Mode;
use vi_book::vim::visual::selection_cols;
use vi_book::{Editor, Position};

#[test]
fn yank_line_with_capital_y_then_paste() {
    let mut ed = Editor::new("hello\nworld");
    ed.handle_keys("Y");
    ed.handle_keys("j");
    ed.handle_keys("p");
    assert_eq!(ed.text(), "hello\nworld\nhello");
}

#[test]
fn ctrl_v_enters_visual_block_mode() {
    let mut ed = Editor::new("abc\ndef");
    ed.handle_keys("<C-v>");
    assert_eq!(ed.mode(), Mode::VisualBlock);
}

#[test]
fn block_delete_removes_a_rectangle() {
    let mut ed = Editor::new("abcd\nefgh\nijkl");
    ed.handle_keys("<C-v>lljjd");
    assert_eq!(ed.text(), "d\nh\nl");
    assert_eq!(ed.mode(), Mode::Normal);
}

#[test]
fn block_yank_captures_the_column() {
    let mut ed = Editor::new("ab\ncd");
    ed.handle_keys("<C-v>jy");
    assert_eq!(ed.mode(), Mode::Normal);
    // the column "a\nc" is captured; pasting it after the cursor reinserts it
    ed.handle_keys("$p");
    assert_eq!(ed.text(), "aba\nc\ncd");
}

#[test]
fn block_change_deletes_the_rectangle_and_inserts_at_the_top() {
    let mut ed = Editor::new("abc\ndef");
    ed.handle_keys("<C-v>jcX");
    ed.handle_keys("<Esc>");
    // block-insert replication across rows is not supported (documented limitation)
    assert_eq!(ed.text(), "Xbc\nef");
}

#[test]
fn selection_cols_charwise_spans_lines() {
    let anchor = Position::new(0, 1);
    let cursor = Position::new(1, 2);
    assert_eq!(
        selection_cols(Mode::Visual, anchor, cursor, 0, 5),
        Some(1..5)
    );
    assert_eq!(
        selection_cols(Mode::Visual, anchor, cursor, 1, 5),
        Some(0..3)
    );
    assert_eq!(selection_cols(Mode::Visual, anchor, cursor, 2, 5), None);
}

#[test]
fn selection_cols_block_is_a_rectangle() {
    let anchor = Position::new(0, 1);
    let cursor = Position::new(2, 3);
    for line in 0..=2 {
        assert_eq!(
            selection_cols(Mode::VisualBlock, anchor, cursor, line, 10),
            Some(1..4)
        );
    }
}

#[test]
fn selection_cols_linewise_covers_whole_line() {
    let anchor = Position::new(0, 2);
    let cursor = Position::new(1, 0);
    assert_eq!(
        selection_cols(Mode::VisualLine, anchor, cursor, 1, 7),
        Some(0..7)
    );
}
