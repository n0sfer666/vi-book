use std::ops::Range;

use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::layout::{Position as ScreenPosition, Rect};
use ratatui::style::{Modifier, Style as TuiStyle};
use ratatui::text::{Line, Span as TuiSpan};
use ratatui::widgets::Paragraph;

use crate::core::position::Position;
use crate::markdown::render::render;
use crate::markdown::rendered::{Rendered, RenderedLine};
use crate::ui::app::App;
use crate::ui::layout::{column_width, text_column};
use crate::ui::theme::Theme;
use crate::ui::view::View;
use crate::ui::{event_map, read_motion};
use crate::vim::key::Key;
use crate::vim::mode::Mode;
use crate::vim::visual;

type Select = (Mode, (usize, usize), (usize, usize));

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
    app.read_select = None;
    app.view = View::Edit;
}

pub fn handle_key(app: &mut App, event: KeyEvent, width: usize) {
    let Some(key) = event_map::to_key(event) else {
        return;
    };
    match key {
        Key::Char('v') => return toggle_select(app, Mode::Visual),
        Key::Char('V') => return toggle_select(app, Mode::VisualLine),
        Key::Ctrl('v') => return toggle_select(app, Mode::VisualBlock),
        Key::Esc => return app.read_select = None,
        Key::Char('y') if app.read_select.is_some() => return yank_selection(app, width),
        Key::Char('Y') => return yank_line(app, width),
        _ => {}
    }
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
    select: Option<Select>,
) -> Vec<Line<'static>> {
    rendered
        .lines
        .iter()
        .enumerate()
        .skip(top)
        .take(height)
        .map(|(idx, line)| {
            let range = select.and_then(|(mode, anchor, cursor)| {
                visual::selection_cols(
                    mode,
                    Position::new(anchor.0, anchor.1),
                    Position::new(cursor.0, cursor.1),
                    idx,
                    line.text().chars().count(),
                )
            });
            rendered_line(line, theme, range.as_ref())
        })
        .collect()
}

fn rendered_line(line: &RenderedLine, theme: &Theme, sel: Option<&Range<usize>>) -> Line<'static> {
    let mut parts: Vec<TuiSpan<'static>> = Vec::new();
    let mut run = String::new();
    let mut run_style = TuiStyle::default();
    let mut col = 0;
    let mut started = false;
    for span in &line.spans {
        let base = theme.tui(span.style);
        for ch in span.text.chars() {
            let mut style = base;
            if sel.is_some_and(|range| range.contains(&col)) {
                style = style.add_modifier(Modifier::REVERSED);
            }
            if started && style != run_style && !run.is_empty() {
                parts.push(TuiSpan::styled(std::mem::take(&mut run), run_style));
            }
            run_style = style;
            started = true;
            run.push(ch);
            col += 1;
        }
    }
    if !run.is_empty() {
        parts.push(TuiSpan::styled(run, run_style));
    }
    Line::from(parts)
}

fn delegates_to_edit(key: Key) -> bool {
    matches!(key, Key::Char(c) if "pPdcxoOaAIsrR".contains(c))
}

fn toggle_select(app: &mut App, mode: Mode) {
    app.read_select = match app.read_select {
        Some((current, _)) if current == mode => None,
        Some((_, anchor)) => Some((mode, anchor)),
        None => Some((mode, app.read_cursor)),
    };
}

fn yank_selection(app: &mut App, width: usize) {
    let rendered = cached_render(app, width);
    if let Some((mode, anchor)) = app.read_select {
        let text = selected_text(&rendered, mode, anchor, app.read_cursor);
        app.editor.yank(text, mode == Mode::VisualLine);
        app.read_select = None;
        app.message = String::from("yanked");
    }
}

fn yank_line(app: &mut App, width: usize) {
    let rendered = cached_render(app, width);
    if let Some(line) = rendered.lines.get(app.read_cursor.0) {
        app.editor.yank(format!("{}\n", line.text()), true);
        app.message = String::from("yanked line");
    }
}

fn selected_text(
    rendered: &Rendered,
    mode: Mode,
    anchor: (usize, usize),
    cursor: (usize, usize),
) -> String {
    let top = anchor.0.min(cursor.0);
    let bottom = anchor.0.max(cursor.0);
    (top..=bottom)
        .filter_map(|line| {
            let chars: Vec<char> = rendered.lines.get(line)?.text().chars().collect();
            let range = visual::selection_cols(
                mode,
                Position::new(anchor.0, anchor.1),
                Position::new(cursor.0, cursor.1),
                line,
                chars.len(),
            )?;
            let from = range.start.min(chars.len());
            let to = range.end.min(chars.len());
            Some(chars[from..to].iter().collect::<String>())
        })
        .collect::<Vec<_>>()
        .join("\n")
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
    let select = app
        .read_select
        .map(|(mode, anchor)| (mode, anchor, app.read_cursor));
    let lines = read_lines(&rendered, app.top, height, &app.theme, select);
    frame.render_widget(Paragraph::new(lines), inner);
    let row = area.y + (app.read_cursor.0 - app.top) as u16;
    let col = left + (app.read_cursor.1 as u16).min(inner.width.saturating_sub(1));
    frame.set_cursor_position(ScreenPosition::new(col, row));
}
