use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Position as ScreenPosition, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use crate::ui::app::App;
use crate::vim::mode::Mode;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let [text_area, status_area, message_area] = Layout::vertical([
        Constraint::Min(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());
    scroll_into_view(app, text_area);
    draw_text(frame, app, text_area);
    draw_status(frame, app, status_area);
    draw_message(frame, app, message_area);
    place_cursor(frame, app, text_area, message_area);
}

fn scroll_into_view(app: &mut App, area: Rect) {
    let height = area.height.max(1) as usize;
    let line = app.editor.cursor().line;
    if line < app.top {
        app.top = line;
    }
    if line >= app.top + height {
        app.top = line + 1 - height;
    }
}

fn draw_text(frame: &mut Frame, app: &App, area: Rect) {
    let text = app.editor.text();
    let lines: Vec<Line> = text
        .split('\n')
        .skip(app.top)
        .take(area.height as usize)
        .map(Line::from)
        .collect();
    frame.render_widget(Paragraph::new(lines), area);
}

fn draw_status(frame: &mut Frame, app: &App, area: Rect) {
    let mode = match app.editor.mode() {
        Mode::Normal => "NORMAL",
        Mode::Insert => "INSERT",
        Mode::Visual => "VISUAL",
        Mode::VisualLine => "V-LINE",
    };
    let dirty = if app.editor.text() != app.saved_text {
        " [+]"
    } else {
        ""
    };
    let cursor = app.editor.cursor();
    let status = format!(
        " {mode}  {}{dirty}  {}:{} ",
        app.path,
        cursor.line + 1,
        cursor.col + 1
    );
    let style = Style::default().add_modifier(Modifier::REVERSED);
    frame.render_widget(Paragraph::new(status).style(style), area);
}

fn draw_message(frame: &mut Frame, app: &App, area: Rect) {
    let content = if let Some(command) = &app.command {
        format!(":{command}")
    } else if let Some(input) = app.editor.search_input() {
        format!("/{input}")
    } else {
        app.message.clone()
    };
    frame.render_widget(Paragraph::new(content), area);
}

fn place_cursor(frame: &mut Frame, app: &App, text_area: Rect, message_area: Rect) {
    if let Some(command) = &app.command {
        let col = message_area.x + 1 + command.chars().count() as u16;
        frame.set_cursor_position(ScreenPosition::new(col, message_area.y));
        return;
    }
    if let Some(input) = app.editor.search_input() {
        let col = message_area.x + 1 + input.chars().count() as u16;
        frame.set_cursor_position(ScreenPosition::new(col, message_area.y));
        return;
    }
    let cursor = app.editor.cursor();
    let row = text_area.y + (cursor.line - app.top) as u16;
    let col = text_area.x + (cursor.col as u16).min(text_area.width.saturating_sub(1));
    frame.set_cursor_position(ScreenPosition::new(col, row));
}
