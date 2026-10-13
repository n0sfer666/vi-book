use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Position as ScreenPosition, Rect};
use ratatui::style::{Modifier, Style as TuiStyle};
use ratatui::text::{Line, Span as TuiSpan};
use ratatui::widgets::Paragraph;

use crate::app::App;
use crate::layout::{column_width, text_column};
use crate::read_state::ReadState;
use crate::theme::Theme;
use crate::view::View;
use crate::{hscroll, overlay, read_view, split};
use v_ide_engine::mode::Mode;
use v_ide_markdown::highlight::highlight;
use v_ide_markdown::style::SourceSpan;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let [body, status_area, message_area] = Layout::vertical([
        Constraint::Min(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());
    let text_area = if app.panel.is_some() {
        let [panel_area, text_area] =
            Layout::horizontal([Constraint::Length(28), Constraint::Min(1)]).areas(body);
        overlay::draw_panel(frame, app, panel_area);
        text_area
    } else {
        body
    };
    let count = app.pane_count();
    let rects = split::split_rects(text_area, count);
    let active = app.workspace.active_window();
    let fallback = ReadState::new();
    for (window, rect) in app.workspace.windows().iter().zip(&rects) {
        if window.id != active {
            let state = app.window_states.get(&window.id).unwrap_or(&fallback);
            split::draw_pane(frame, &app.theme, &window.buf.doc, state, *rect, app.wrap);
        }
    }
    for rect in rects.iter().take(count.saturating_sub(1)) {
        split::draw_divider(frame, text_area, rect.x + rect.width);
    }
    let active_rect = rects[app.workspace.active_index()];
    app.text_area = active_rect;
    let text = app.workspace.current().editor.text();
    if app.view == View::Read {
        read_view::draw(frame, app, active_rect);
    } else {
        scroll_into_view(app, active_rect);
        let col = app.workspace.current().editor.cursor().col;
        app.left = hscroll::scroll(app.left, col, column_width(active_rect));
        draw_text(frame, app, &text, active_rect);
    }
    draw_status(frame, app, status_area);
    draw_message(frame, app, message_area);
    if app.help {
        overlay::draw_help(frame, app, text_area);
        return;
    }
    if let Some(menu) = &app.menu {
        overlay::draw_menu(frame, menu, text_area);
        return;
    }
    if app.buffer_list.is_some() {
        overlay::draw_buffer_list(frame, app, text_area);
        return;
    }
    if app.picker.is_some() {
        overlay::draw_picker(frame, app, text_area);
        return;
    }
    if app.view == View::Edit {
        place_cursor(frame, app, active_rect, message_area);
    } else if app.command.is_some() || app.workspace.current().editor.search_input().is_some() {
        place_bottom_cursor(frame, app, message_area);
    }
}

fn scroll_into_view(app: &mut App, area: Rect) {
    let height = area.height.max(1) as usize;
    let line = app.workspace.current().editor.cursor().line;
    if line < app.top {
        app.top = line;
    }
    if line >= app.top + height {
        app.top = line + 1 - height;
    }
}

fn draw_text(frame: &mut Frame, app: &App, text: &str, area: Rect) {
    let highlights = highlight(text);
    let editor = &app.workspace.current().editor;
    let selecting = !matches!(editor.mode(), Mode::Normal | Mode::Insert);
    let anchor = editor.visual_anchor();
    let cursor = editor.cursor();
    let lines: Vec<Line> = text
        .split('\n')
        .enumerate()
        .skip(app.top)
        .take(area.height as usize)
        .map(|(idx, line)| {
            let selection = selecting
                .then(|| {
                    v_ide_engine::visual::selection_cols(
                        editor.mode(),
                        anchor,
                        cursor,
                        idx,
                        line.chars().count(),
                    )
                })
                .flatten();
            styled_line(
                line,
                highlights.get(idx).map_or(&[][..], Vec::as_slice),
                &app.theme,
                selection.as_ref(),
            )
        })
        .collect();
    let inner = text_column(area);
    frame.render_widget(Paragraph::new(lines).scroll((0, app.left as u16)), inner);
    let source: Vec<&str> = text.split('\n').collect();
    hscroll::markers(frame, inner, app.left, area.height as usize, |row| {
        source.get(app.top + row).map(|line| line.chars().count())
    });
}

pub(crate) fn styled_line(
    line: &str,
    spans: &[SourceSpan],
    theme: &Theme,
    selection: Option<&std::ops::Range<usize>>,
) -> Line<'static> {
    let chars: Vec<char> = line.chars().collect();
    let mut parts: Vec<TuiSpan<'static>> = Vec::new();
    let mut run = String::new();
    let mut run_style = TuiStyle::default();
    for (col, ch) in chars.iter().enumerate() {
        let mut style = spans
            .iter()
            .filter(|span| span.start <= col && col < span.end)
            .min_by_key(|span| span.end - span.start)
            .map_or_else(TuiStyle::default, |span| theme.tui(span.style));
        if selection.is_some_and(|range| range.contains(&col)) {
            style = style.add_modifier(Modifier::REVERSED);
        }
        if style != run_style && !run.is_empty() {
            parts.push(TuiSpan::styled(std::mem::take(&mut run), run_style));
        }
        run_style = style;
        run.push(*ch);
    }
    if !run.is_empty() {
        parts.push(TuiSpan::styled(run, run_style));
    }
    Line::from(parts)
}

fn draw_status(frame: &mut Frame, app: &App, area: Rect) {
    let doc = app.workspace.current();
    let mode = if app.view == View::Read {
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

fn draw_message(frame: &mut Frame, app: &App, area: Rect) {
    let content = if let Some(command) = &app.command {
        format!(":{command}")
    } else if let Some(input) = app.workspace.current().editor.search_input() {
        format!("/{input}")
    } else {
        app.message.clone()
    };
    frame.render_widget(Paragraph::new(content), area);
}

fn place_bottom_cursor(frame: &mut Frame, app: &App, message_area: Rect) {
    let len = app
        .command
        .as_deref()
        .or(app.workspace.current().editor.search_input())
        .map_or(0, |s| s.chars().count());
    let col = message_area.x + 1 + len as u16;
    frame.set_cursor_position(ScreenPosition::new(col, message_area.y));
}

fn place_cursor(frame: &mut Frame, app: &App, text_area: Rect, message_area: Rect) {
    let editor = &app.workspace.current().editor;
    if app.command.is_some() || editor.search_input().is_some() {
        place_bottom_cursor(frame, app, message_area);
        return;
    }
    let inner = text_column(text_area);
    let cursor = editor.cursor();
    let row = inner.y + (cursor.line - app.top) as u16;
    let col =
        inner.x + (cursor.col.saturating_sub(app.left) as u16).min(inner.width.saturating_sub(1));
    frame.set_cursor_position(ScreenPosition::new(col, row));
}
