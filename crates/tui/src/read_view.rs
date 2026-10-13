use std::ops::Range;

use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::layout::{Position as ScreenPosition, Rect};
use ratatui::style::{Modifier, Style as TuiStyle};
use ratatui::text::{Line, Span as TuiSpan};
use ratatui::widgets::Paragraph;

use crate::app::App;
use crate::layout::{column_width, text_column};
use crate::theme::Theme;
use crate::view::View;
use crate::{clip_sync, event_map, hscroll, read_motion};
use v_ide_engine::mode::Mode;
use v_ide_engine::visual;
use v_ide_keys::key::Key;
use v_ide_markdown::blocks::{self, BlockKind};
use v_ide_markdown::render::render_with;
use v_ide_markdown::rendered::{Rendered, RenderedLine};
use v_ide_text::position::Position;

type Select = (Mode, (usize, usize), (usize, usize));

pub fn cached_render(app: &mut App, width: usize) -> Rendered {
    let text = app.workspace.current().editor.text();
    let wrap = app.wrap;
    let hit = matches!(&app.ui.read.read_cache, Some((t, w, m, _)) if t == &text && *w == width && *m == wrap);
    if !hit {
        let rendered = render_with(&text, width, wrap);
        app.ui.read.read_cache = Some((text, width, wrap, rendered));
    }
    app.ui
        .read
        .read_cache
        .as_ref()
        .map(|(_, _, _, r)| r.clone())
        .unwrap_or_default()
}

pub fn enter(app: &mut App, width: usize) {
    let rendered = cached_render(app, width);
    let idx = app.workspace.current().editor.text_index();
    app.ui.read.read_cursor = rendered.to_rendered(idx);
    app.ui.read.view = View::Read;
}

pub fn leave(app: &mut App, width: usize) {
    let rendered = cached_render(app, width);
    if let Some(idx) = source_at(
        &rendered,
        app.ui.read.read_cursor.0,
        app.ui.read.read_cursor.1,
    ) {
        app.workspace.current_mut().editor.set_cursor_index(idx);
    }
    app.ui.read_select = None;
    app.ui.read.view = View::Edit;
}

pub fn handle_key(app: &mut App, event: KeyEvent, width: usize) {
    let Some(key) = event_map::to_key(event) else {
        return;
    };
    match key {
        Key::Char('v') => return toggle_select(app, Mode::Visual),
        Key::Char('V') => return toggle_select(app, Mode::VisualLine),
        Key::Ctrl('v') => return toggle_select(app, Mode::VisualBlock),
        Key::Esc => return app.ui.read_select = None,
        Key::Char('y') if app.ui.read_select.is_some() => return yank_selection(app, width),
        Key::Char('Y') => return yank_line(app, width),
        _ => {}
    }
    if delegates_to_edit(key) {
        leave(app, width);
        clip_sync::editor_key(app, key);
        return;
    }
    let rendered = cached_render(app, width);
    let last_line = rendered.lines.len().saturating_sub(1);
    let (line, col) = app.ui.read.read_cursor;
    match key {
        Key::Char('j') | Key::Down => app.ui.read.read_cursor = clamp(&rendered, line + 1, col),
        Key::Char('k') | Key::Up => {
            app.ui.read.read_cursor = clamp(&rendered, line.saturating_sub(1), col)
        }
        Key::Char('h') | Key::Left => {
            app.ui.read.read_cursor = clamp(&rendered, line, col.saturating_sub(1))
        }
        Key::Char('l') | Key::Right => app.ui.read.read_cursor = clamp(&rendered, line, col + 1),
        Key::Char('w') => {
            app.ui.read.read_cursor = read_motion::word_forward(&rendered, (line, col))
        }
        Key::Char('b') => app.ui.read.read_cursor = read_motion::word_back(&rendered, (line, col)),
        Key::Char('e') => app.ui.read.read_cursor = read_motion::word_end(&rendered, (line, col)),
        Key::Char('W') => {
            app.ui.read.read_cursor = read_motion::element_forward(&rendered, (line, col))
        }
        Key::Char('B') => {
            app.ui.read.read_cursor = read_motion::element_back(&rendered, (line, col))
        }
        Key::Char('g') => {
            if app.ui.read.read_pending_g {
                app.ui.read.read_cursor = (0, 0);
            }
            app.ui.read.read_pending_g = !app.ui.read.read_pending_g;
            return;
        }
        Key::Char('G') => app.ui.read.read_cursor = clamp(&rendered, last_line, 0),
        Key::Char('0') => app.ui.read.read_cursor = (line, 0),
        Key::Char('$') => app.ui.read.read_cursor = clamp(&rendered, line, usize::MAX),
        Key::Char('i') => leave(app, width),
        Key::Ctrl('t') => leave(app, width),
        Key::Enter => {
            if let Some(idx) = source_at(&rendered, line, col) {
                app.workspace.current_mut().editor.set_cursor_index(idx);
            }
            if let Some(code) = code_block_at(
                &app.workspace.current().editor.text(),
                app.workspace.current().editor.cursor().line,
            ) {
                app.workspace.current_mut().editor.yank(code, false);
                clip_sync::sync_out(app);
                app.ui.message = String::from("copied code block to clipboard");
            } else {
                crate::link_follow::follow_at_cursor(app);
            }
        }
        _ => {}
    }
    app.ui.read.read_pending_g = false;
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
    app.ui.read_select = match app.ui.read_select {
        Some((current, _)) if current == mode => None,
        Some((_, anchor)) => Some((mode, anchor)),
        None => Some((mode, app.ui.read.read_cursor)),
    };
}

fn yank_selection(app: &mut App, width: usize) {
    let rendered = cached_render(app, width);
    if let Some((mode, anchor)) = app.ui.read_select {
        let text = selected_text(&rendered, mode, anchor, app.ui.read.read_cursor);
        app.workspace
            .current_mut()
            .editor
            .yank(text, mode == Mode::VisualLine);
        clip_sync::sync_out(app);
        app.ui.read_select = None;
        app.ui.message = String::from("copied to clipboard");
    }
}

fn yank_line(app: &mut App, width: usize) {
    let rendered = cached_render(app, width);
    if let Some(line) = rendered.lines.get(app.ui.read.read_cursor.0) {
        let text = line.text();
        app.workspace
            .current_mut()
            .editor
            .yank(format!("{text}\n"), true);
        clip_sync::sync_out(app);
        app.ui.message = String::from("copied line to clipboard");
    }
}

fn code_block_at(text: &str, src_line: usize) -> Option<String> {
    blocks::analyze(text).into_iter().find_map(|block| {
        if let BlockKind::Code { lines, .. } = block.kind
            && src_line >= block.start_line
            && src_line <= block.end_line
        {
            let code = lines
                .iter()
                .map(|line| line.iter().map(|atom| atom.ch).collect::<String>())
                .collect::<Vec<_>>()
                .join("\n");
            return Some(code);
        }
        None
    })
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
    app.ui.read.read_cursor = clamp(
        &rendered,
        app.ui.read.read_cursor.0,
        app.ui.read.read_cursor.1,
    );
    let height = area.height.max(1) as usize;
    if app.ui.read.read_cursor.0 < app.ui.read.top {
        app.ui.read.top = app.ui.read.read_cursor.0;
    }
    if app.ui.read.read_cursor.0 >= app.ui.read.top + height {
        app.ui.read.top = app.ui.read.read_cursor.0 + 1 - height;
    }
    app.ui.left = hscroll::scroll(app.ui.left, app.ui.read.read_cursor.1, width);
    let inner = text_column(area);
    let select = app
        .ui
        .read_select
        .map(|(mode, anchor)| (mode, anchor, app.ui.read.read_cursor));
    let lines = read_lines(&rendered, app.ui.read.top, height, &app.theme, select);
    frame.render_widget(Paragraph::new(lines).scroll((0, app.ui.left as u16)), inner);
    hscroll::markers(frame, inner, app.ui.left, height, |row| {
        rendered
            .lines
            .get(app.ui.read.top + row)
            .map(|line| line.text().chars().count())
    });
    let row = area.y + (app.ui.read.read_cursor.0 - app.ui.read.top) as u16;
    let col = inner.x
        + (app.ui.read.read_cursor.1.saturating_sub(app.ui.left) as u16)
            .min(inner.width.saturating_sub(1));
    frame.set_cursor_position(ScreenPosition::new(col, row));
}
