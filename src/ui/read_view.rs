use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::layout::{Position as ScreenPosition, Rect};
use ratatui::text::{Line, Span as TuiSpan};
use ratatui::widgets::Paragraph;

use crate::markdown::render::render;
use crate::markdown::rendered::Rendered;
use crate::ui::app::App;
use crate::ui::event_map;
use crate::ui::view::View;
use crate::vim::key::Key;

const MAX_COLUMN: usize = 90;

pub fn column_width(area: Rect) -> usize {
    (area.width as usize).clamp(1, MAX_COLUMN)
}

pub fn enter(app: &mut App, width: usize) {
    let rendered = render(&app.editor.text(), width);
    let idx = app.editor.text_index();
    app.read_cursor = rendered.to_rendered(idx);
    app.view = View::Read;
}

pub fn leave(app: &mut App, width: usize) {
    let rendered = render(&app.editor.text(), width);
    if let Some(idx) = source_at(&rendered, app.read_cursor.0, app.read_cursor.1) {
        app.editor.set_cursor_index(idx);
    }
    app.view = View::Edit;
}

pub fn handle_key(app: &mut App, event: KeyEvent, width: usize) {
    let Some(key) = event_map::to_key(event) else {
        return;
    };
    let rendered = render(&app.editor.text(), width);
    let last_line = rendered.lines.len().saturating_sub(1);
    let (line, col) = app.read_cursor;
    match key {
        Key::Char('j') => app.read_cursor = clamp(&rendered, line + 1, col),
        Key::Char('k') => app.read_cursor = clamp(&rendered, line.saturating_sub(1), col),
        Key::Char('h') => app.read_cursor = clamp(&rendered, line, col.saturating_sub(1)),
        Key::Char('l') => app.read_cursor = clamp(&rendered, line, col + 1),
        Key::Char('g') => {
            if app.read_pending_g {
                app.read_cursor = (0, 0);
            }
            app.read_pending_g = !app.read_pending_g;
            return;
        }
        Key::Char('G') => app.read_cursor = clamp(&rendered, last_line, 0),
        Key::Char('0') => app.read_cursor = (line, 0),
        Key::Char('$') => app.read_cursor = clamp(&rendered, line, usize::MAX),
        Key::Char('i') => leave(app, width),
        Key::Ctrl('t') => leave(app, width),
        _ => {}
    }
    app.read_pending_g = false;
}

fn clamp(rendered: &Rendered, line: usize, col: usize) -> (usize, usize) {
    let last = rendered.lines.len().saturating_sub(1);
    let line = line.min(last);
    let len = rendered
        .lines
        .get(line)
        .map_or(0, |l| l.text().chars().count());
    (line, col.min(len.saturating_sub(1)))
}

fn source_at(rendered: &Rendered, line: usize, col: usize) -> Option<usize> {
    let len = rendered.lines.get(line)?.text().chars().count();
    (col..len)
        .chain((0..col).rev())
        .find_map(|c| rendered.to_source(line, c))
        .or(Some(0))
}

pub fn draw(frame: &mut Frame, app: &mut App, area: Rect) {
    let width = column_width(area);
    let rendered = render(&app.editor.text(), width);
    app.read_cursor = clamp(&rendered, app.read_cursor.0, app.read_cursor.1);
    let height = area.height.max(1) as usize;
    if app.read_cursor.0 < app.top {
        app.top = app.read_cursor.0;
    }
    if app.read_cursor.0 >= app.top + height {
        app.top = app.read_cursor.0 + 1 - height;
    }
    let left = area.x + ((area.width as usize).saturating_sub(width) / 2) as u16;
    let lines: Vec<Line> = rendered
        .lines
        .iter()
        .skip(app.top)
        .take(height)
        .map(|line| {
            Line::from(
                line.spans
                    .iter()
                    .map(|span| TuiSpan::styled(span.text.clone(), app.theme.tui(span.style)))
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    let inner = Rect {
        x: left,
        y: area.y,
        width: width.min(area.width as usize) as u16,
        height: area.height,
    };
    frame.render_widget(Paragraph::new(lines), inner);
    let row = area.y + (app.read_cursor.0 - app.top) as u16;
    let col = left + (app.read_cursor.1 as u16).min(inner.width.saturating_sub(1));
    frame.set_cursor_position(ScreenPosition::new(col, row));
}
