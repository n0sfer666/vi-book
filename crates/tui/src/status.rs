use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style as TuiStyle};
use ratatui::widgets::Paragraph;

use crate::app::App;
use crate::view::View;
use v_ide_engine::mode::Mode;

pub(crate) fn draw_status(frame: &mut Frame, app: &App, area: Rect) {
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
    let cursor = doc.editor.cursor();
    let status = format!(
        " {mode}  {}{dirty}  {}:{} ",
        doc.name,
        cursor.line + 1,
        cursor.col + 1
    );
    let style = TuiStyle::default().add_modifier(Modifier::REVERSED);
    frame.render_widget(Paragraph::new(status).style(style), area);
}

pub(crate) fn draw_message(frame: &mut Frame, app: &App, area: Rect) {
    let content = if let Some(command) = &app.ui.command {
        format!(":{command}")
    } else if let Some(input) = app.workspace.current().editor.search_input() {
        format!("/{input}")
    } else {
        app.ui.message.clone()
    };
    frame.render_widget(Paragraph::new(content), area);
}
