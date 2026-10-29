mod common;

use common::feed::{app, press};
use v_ide_tui::app::App;

fn find(app: &mut App, needle: &str) -> (u16, u16, Option<(u16, u16)>) {
    let frame = common::draw(app, 40, 8);
    let (y, x) = (0..8)
        .find_map(|y| {
            let row = frame.grid.row_text(y);
            row.find(needle)
                .map(|at| (y, u16::try_from(row[..at].chars().count()).unwrap_or(0)))
        })
        .expect("needle on screen");
    (x, y, frame.cursor)
}

#[test]
fn cursor_after_wide_char_counts_its_width() {
    let mut app = app("日");
    press(&mut app, "A");
    let (x, y, cursor) = find(&mut app, "日");
    assert_eq!(cursor, Some((x + 2, y)));
}

#[test]
fn pending_key_after_wide_char_keeps_the_glyph() {
    let mut app = app("日");
    press(&mut app, "Aj");
    let (x, y, cursor) = find(&mut app, "日j");
    assert_eq!(cursor, Some((x + 2, y)));
}
