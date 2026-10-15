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

pub(crate) fn place_cursor(frame: &mut Frame, app: &App, text_area: Rect, message_area: Rect) {
    let editor = &app.workspace.current().editor;
    if app.ui.command.is_some() || editor.search_input().is_some() {
        place_bottom_cursor(frame, app, message_area);
        return;
    }
    let inner = text_column(text_area);
    let cursor = editor.cursor();
    let row = inner.y + (cursor.line - app.ui.read.top) as u16;
    let col =
        inner.x + (cursor.col.saturating_sub(app.ui.left) as u16).min(inner.w.saturating_sub(1));
    frame.cursor = Some((col, row));
}
