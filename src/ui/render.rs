use ratatui::layout::{Constraint, Layout, Position as ScreenPosition, Rect};
use ratatui::style::{Modifier, Style as TuiStyle};
use ratatui::text::{Line, Span as TuiSpan};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::markdown::highlight::highlight;
use crate::markdown::style::SourceSpan;
use crate::ui::app::App;
use crate::ui::view::View;
use crate::ui::{overlay, read_view};
use crate::vim::mode::Mode;

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
    app.text_area = text_area;
    let text = app.editor.text();
    if app.view == View::Read {
        read_view::draw(frame, app, text_area);
    } else {
        scroll_into_view(app, text_area);
        draw_text(frame, app, &text, text_area);
    }
    draw_status(frame, app, &text, status_area);
    draw_message(frame, app, message_area);
    if app.help {
        overlay::draw_help(frame, app, text_area);
        return;
    }
    if let Some(menu) = &app.menu {
        overlay::draw_menu(frame, menu, text_area);
        return;
    }
    if app.picker.is_some() {
        overlay::draw_picker(frame, app, text_area);
        return;
    }
    if app.view == View::Edit {
        place_cursor(frame, app, text_area, message_area);
    } else if app.command.is_some() || app.editor.search_input().is_some() {
        place_bottom_cursor(frame, app, message_area);
    }
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

fn draw_text(frame: &mut Frame, app: &App, text: &str, area: Rect) {
    let highlights = highlight(text);
    let lines: Vec<Line> = text
        .split('\n')
        .enumerate()
        .skip(app.top)
        .take(area.height as usize)
        .map(|(idx, line)| {
            styled_line(
                line,
                highlights.get(idx).map_or(&[][..], Vec::as_slice),
                app,
            )
        })
        .collect();
    frame.render_widget(Paragraph::new(lines), area);
}

fn styled_line<'l>(line: &'l str, spans: &[SourceSpan], app: &App) -> Line<'l> {
    let chars: Vec<char> = line.chars().collect();
    let mut parts: Vec<TuiSpan<'l>> = Vec::new();
    let mut run = String::new();
    let mut run_style = TuiStyle::default();
    for (col, ch) in chars.iter().enumerate() {
        let style = spans
            .iter()
            .filter(|span| span.start <= col && col < span.end)
            .min_by_key(|span| span.end - span.start)
            .map_or_else(TuiStyle::default, |span| app.theme.tui(span.style));
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

fn draw_status(frame: &mut Frame, app: &App, text: &str, area: Rect) {
    let mode = if app.view == View::Read {
        "READ"
    } else {
        match app.editor.mode() {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
            Mode::Visual => "VISUAL",
            Mode::VisualLine => "V-LINE",
        }
    };
    let dirty = if text != app.saved_text { " [+]" } else { "" };
    let cursor = app.editor.cursor();
    let status = format!(
        " {mode}  {}{dirty}  {}:{} ",
        app.path,
        cursor.line + 1,
        cursor.col + 1
    );
    let style = TuiStyle::default().add_modifier(Modifier::REVERSED);
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

fn place_bottom_cursor(frame: &mut Frame, app: &App, message_area: Rect) {
    let len = app
        .command
        .as_deref()
        .or(app.editor.search_input())
        .map_or(0, |s| s.chars().count());
    let col = message_area.x + 1 + len as u16;
    frame.set_cursor_position(ScreenPosition::new(col, message_area.y));
}

fn place_cursor(frame: &mut Frame, app: &App, text_area: Rect, message_area: Rect) {
    if app.command.is_some() || app.editor.search_input().is_some() {
        place_bottom_cursor(frame, app, message_area);
        return;
    }
    let cursor = app.editor.cursor();
    let row = text_area.y + (cursor.line - app.top) as u16;
    let col = text_area.x + (cursor.col as u16).min(text_area.width.saturating_sub(1));
    frame.set_cursor_position(ScreenPosition::new(col, row));
}
