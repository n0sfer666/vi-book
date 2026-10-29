use unicode_width::UnicodeWidthStr;
use v_ide_keys::{Code, MapMode, Mods};
use v_ide_view::Rect;

use crate::app::App;
use crate::frame::Frame;
use crate::layout::text_column;

pub(crate) fn place_bottom_cursor(frame: &mut Frame, app: &App, message_area: Rect) {
    let len = app
        .ui
        .command
        .as_deref()
        .or(app.workspace.current().editor.search_input())
        .map_or(0, |s| s.chars().count());
    let col = message_area.x + 1 + len as u16;
    frame.cursor = Some((col, message_area.y));
}

pub(crate) fn place_cursor(
    frame: &mut Frame,
    app: &App,
    text: &str,
    text_area: Rect,
    message_area: Rect,
) {
    let editor = &app.workspace.current().editor;
    if app.ui.command.is_some() || editor.search_input().is_some() {
        place_bottom_cursor(frame, app, message_area);
        return;
    }
    let inner = text_column(text_area);
    let cursor = editor.cursor();
    let row = inner.y + (cursor.line - app.ui.read.top) as u16;
    let shown: String = text
        .split('\n')
        .nth(cursor.line)
        .unwrap_or_default()
        .chars()
        .take(cursor.col)
        .skip(app.ui.left)
        .collect();
    let width = u16::try_from(shown.width()).unwrap_or(u16::MAX);
    let col = inner.x + width.min(inner.w.saturating_sub(1));
    if editor.map_mode() == Some(MapMode::Insert) {
        put_pending(frame, app, col, row, inner.x + inner.w - col);
    }
    frame.cursor = Some((col, row));
}

fn put_pending(frame: &mut Frame, app: &App, col: u16, row: u16, max_w: u16) {
    let Some(key) = app.typeahead.pending().last() else {
        return;
    };
    let area = frame.grid.area();
    if max_w == 0 || col >= area.w || row >= area.h {
        return;
    }
    if let (Mods::NONE, Code::Char(c)) = (key.mods(), key.code()) {
        let style = frame.grid.cell(col, row).style;
        frame.grid.put_str(col, row, &c.to_string(), style, max_w);
    }
}
