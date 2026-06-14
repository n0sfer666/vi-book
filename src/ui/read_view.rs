use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::layout::{Position as ScreenPosition, Rect};
use ratatui::text::{Line, Span as TuiSpan};
use ratatui::widgets::Paragraph;

use crate::markdown::render::render;
use crate::markdown::rendered::Rendered;
use crate::ui::app::App;
use crate::ui::layout::{column_width, text_column};
use crate::ui::theme::Theme;
use crate::ui::view::View;
use crate::ui::{event_map, read_motion};
use crate::vim::key::Key;

pub fn cached_render(app: &mut App, width: usize) -> Rendered {
    let text = app.editor.text();
    let hit = matches!(&app.read_cache, Some((t, w, _)) if t == &text && *w == width);
    if !hit {
        let rendered = render(&text, width);
        app.read_cache = Some((text, width, rendered));
    }
    app.read_cache
        .as_ref()
        .map(|(_, _, r)| r.clone())
        .unwrap_or_default()
}

pub fn enter(app: &mut App, width: usize) {
    let rendered = cached_render(app, width);
    let idx = app.editor.text_index();
    app.read_cursor = rendered.to_rendered(idx);
    app.view = View::Read;
}

pub fn leave(app: &mut App, width: usize) {
    let rendered = cached_render(app, width);
    if let Some(idx) = source_at(&rendered, app.read_cursor.0, app.read_cursor.1) {
        app.editor.set_cursor_index(idx);
    }
    app.view = View::Edit;
}

pub fn handle_key(app: &mut App, event: KeyEvent, width: usize) {
    let Some(key) = event_map::to_key(event) else {
        return;
    };
    if delegates_to_edit(key) {
        leave(app, width);
        app.editor.handle_key(key);
        return;
    }
    let rendered = cached_render(app, width);
    let last_line = rendered.lines.len().saturating_sub(1);
    let (line, col) = app.read_cursor;
    match key {
        Key::Char('j') | Key::Down => app.read_cursor = clamp(&rendered, line + 1, col),
        Key::Char('k') | Key::Up => app.read_cursor = clamp(&rendered, line.saturating_sub(1), col),
        Key::Char('h') | Key::Left => {
            app.read_cursor = clamp(&rendered, line, col.saturating_sub(1))
        }
        Key::Char('l') | Key::Right => app.read_cursor = clamp(&rendered, line, col + 1),
        Key::Char('w') => app.read_cursor = read_motion::word_forward(&rendered, (line, col)),
        Key::Char('b') => app.read_cursor = read_motion::word_back(&rendered, (line, col)),
        Key::Char('e') => app.read_cursor = read_motion::word_end(&rendered, (line, col)),
        Key::Char('W') => app.read_cursor = read_motion::element_forward(&rendered, (line, col)),
        Key::Char('B') => app.read_cursor = read_motion::element_back(&rendered, (line, col)),
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
        Key::Enter => {
            if let Some(idx) = source_at(&rendered, line, col) {
                app.editor.set_cursor_index(idx);
            }
            crate::ui::link_follow::follow_at_cursor(app);
        }
        _ => {}
    }
    app.read_pending_g = false;
}

pub(crate) fn read_lines(
    rendered: &Rendered,
    top: usize,
    height: usize,
    theme: &Theme,
) -> Vec<Line<'static>> {
    rendered
        .lines
        .iter()
        .skip(top)
        .take(height)
        .map(|line| {
            Line::from(
                line.spans
                    .iter()
                    .map(|span| TuiSpan::styled(span.text.clone(), theme.tui(span.style)))
                    .collect::<Vec<_>>(),
            )
        })
        .collect()
}

fn delegates_to_edit(key: Key) -> bool {
    matches!(key, Key::Ctrl('v')) || matches!(key, Key::Char(c) if "vVpPyYdcxoOaAIsrR".contains(c))
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
    let rendered = cached_render(app, width);
    app.read_cursor = clamp(&rendered, app.read_cursor.0, app.read_cursor.1);
    let height = area.height.max(1) as usize;
    if app.read_cursor.0 < app.top {
        app.top = app.read_cursor.0;
    }
    if app.read_cursor.0 >= app.top + height {
        app.top = app.read_cursor.0 + 1 - height;
    }
    let inner = text_column(area);
    let left = inner.x;
    let lines = read_lines(&rendered, app.top, height, &app.theme);
    frame.render_widget(Paragraph::new(lines), inner);
    let row = area.y + (app.read_cursor.0 - app.top) as u16;
    let col = left + (app.read_cursor.1 as u16).min(inner.width.saturating_sub(1));
    frame.set_cursor_position(ScreenPosition::new(col, row));
}
