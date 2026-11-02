use v_ide_view::{Grid, Rect};

use crate::app::App;
use crate::frame::Frame;
use crate::layout::{column_width, text_column};
use crate::read_state::ReadState;
use crate::view::View;
use crate::{cursor, hscroll, overlay, read_draw, split, status};
use v_ide_editor::options::{Access, Name, Value};
use v_ide_engine::mode::Mode;
use v_ide_markdown::highlight::highlight;

pub(crate) use crate::text_draw::{Line, put_line, styled_line};

const PANEL_WIDTH: u16 = 28;

pub fn draw(frame: &mut Frame, app: &mut App) {
    draw_editor(frame, app);
    crate::message_draw::draw(frame, app);
}

fn draw_editor(frame: &mut Frame, app: &mut App) {
    let area = frame.grid.area();
    app.ui.screen = area;
    let body_h = area.h.saturating_sub(2);
    let body = Rect { h: body_h, ..area };
    let status_area = Rect {
        y: body_h,
        h: 1,
        ..area
    };
    let message_area = Rect {
        y: body_h + 1,
        h: 1,
        ..area
    };
    let text_area = if app.ui.panel.is_some() {
        let panel_w = PANEL_WIDTH.min(body.w.saturating_sub(1));
        overlay::draw_panel(&mut frame.grid, app, Rect { w: panel_w, ..body });
        Rect {
            x: body.x + panel_w,
            w: body.w - panel_w,
            ..body
        }
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
            let doc = &window.buf.doc;
            let rect = crate::number::draw(
                &mut frame.grid,
                *rect,
                state.top,
                doc.editor.text().split('\n').count(),
                state.view == View::Edit && window.options.get(Name::Number) == Value::Bool(true),
            );
            split::draw_pane_with_numbers(
                &mut frame.grid,
                &app.theme,
                doc,
                state,
                rect,
                app.wrap(),
                app.read_number,
            );
        }
    }
    for rect in rects.iter().take(count.saturating_sub(1)) {
        split::draw_divider(&mut frame.grid, text_area, rect.x + rect.w);
    }
    let mut active_rect = rects[app.workspace.active_index()];
    app.ui.text_area = active_rect;
    let text = app.workspace.current().editor.text();
    if app.ui.read.view == View::Read {
        read_draw::draw(frame, app, active_rect);
    } else {
        scroll_into_view(app, active_rect);
        active_rect = crate::number::draw(
            &mut frame.grid,
            active_rect,
            app.ui.read.top,
            text.split('\n').count(),
            app.workspace.option(Name::Number, Access::Local) == Value::Bool(true),
        );
        let col = app.workspace.current().editor.cursor().col;
        app.ui.left = hscroll::scroll(app.ui.left, col, column_width(active_rect));
        draw_text(&mut frame.grid, app, &text, active_rect);
    }
    status::draw_status(&mut frame.grid, app, status_area);
    status::draw_message(&mut frame.grid, app, message_area);
    if app.ui.help {
        overlay::draw_help(&mut frame.grid, app, text_area);
        return;
    }
    if let Some(menu) = &app.ui.menu {
        overlay::draw_menu(&mut frame.grid, menu, text_area);
        return;
    }
    if app.ui.buffer_list.is_some() {
        overlay::draw_buffer_list(frame, app, text_area);
        return;
    }
    if app.ui.picker.is_some() {
        overlay::draw_picker(frame, app, text_area);
        return;
    }
    if app.ui.read.view == View::Edit {
        cursor::place_cursor(frame, app, &text, active_rect, message_area);
    } else if app.ui.command.is_some() || app.workspace.current().editor.search_input().is_some() {
        cursor::place_bottom_cursor(frame, app, message_area);
    }
}

fn scroll_into_view(app: &mut App, area: Rect) {
    let height = usize::from(area.h.max(1));
    let line = app.workspace.current().editor.cursor().line;
    if line < app.ui.read.top {
        app.ui.read.top = line;
    }
    if line >= app.ui.read.top + height {
        app.ui.read.top = line + 1 - height;
    }
}

fn draw_text(grid: &mut Grid, app: &App, text: &str, area: Rect) {
    let highlights = highlight(text);
    let editor = &app.workspace.current().editor;
    let selecting = !matches!(editor.mode(), Mode::Normal | Mode::Insert);
    let anchor = editor.visual_anchor();
    let cursor = editor.cursor();
    let inner = text_column(area);
    let rows = text
        .split('\n')
        .enumerate()
        .skip(app.ui.read.top)
        .take(usize::from(area.h));
    for (row, (idx, line)) in (inner.y..).zip(rows) {
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
        let styled = styled_line(
            line,
            highlights.get(idx).map_or(&[][..], Vec::as_slice),
            &app.theme,
            selection.as_ref(),
        );
        put_line(grid, inner.x, row, &styled, app.ui.left, inner.w);
    }
    let source: Vec<&str> = text.split('\n').collect();
    hscroll::markers(grid, inner, app.ui.left, usize::from(area.h), |row| {
        source
            .get(app.ui.read.top + row)
            .map(|line| line.chars().count())
    });
}
