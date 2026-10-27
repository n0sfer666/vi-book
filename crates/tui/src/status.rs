use v_ide_view::{Grid, Rect, Style};

use crate::app::App;
use crate::disk_mark;
use crate::view::View;
use v_ide_engine::mode::Mode;

pub(crate) fn draw_status(grid: &mut Grid, app: &App, area: Rect) {
    let doc = app.workspace.current();
    let mode = if app.ui.read.view == View::Read {
        "READ"
    } else {
        match doc.editor.mode() {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
            Mode::Visual => "VISUAL",
            Mode::VisualLine => "V-LINE",
            Mode::VisualBlock => "V-BLOCK",
        }
    };
    let dirty = if doc.is_dirty() { " [+]" } else { "" };
    let mark = disk_mark::suffix(doc.disk_mark());
    let cursor = doc.editor.cursor();
    let status = format!(
        " {mode}  {}{dirty}{mark}  {}:{} ",
        doc.name,
        cursor.line + 1,
        cursor.col + 1
    );
    let style = Style::default().reverse();
    grid.fill(area, style);
    grid.put_str(area.x, area.y, &status, style, area.w);
}

pub(crate) fn draw_message(grid: &mut Grid, app: &App, area: Rect) {
    let content = if let Some(command) = &app.ui.command {
        format!(":{command}")
    } else if let Some(input) = app.workspace.current().editor.search_input() {
        format!("/{input}")
    } else {
        app.ui.message.clone()
    };
    grid.put_str(area.x, area.y, &content, Style::default(), area.w);
}
