use std::ops::Range;

use v_ide_view::{Rect, Style as CellStyle};

use crate::app::App;
use crate::frame::Frame;
use crate::hscroll;
use crate::layout::{column_width, text_column};
use crate::read_view::{cached_render, clamp};
use crate::render::{Line, put_line};
use crate::theme::Theme;
use v_ide_engine::mode::Mode;
use v_ide_engine::visual;
use v_ide_markdown::rendered::{Rendered, RenderedLine};
use v_ide_text::position::Position;

type Select = (Mode, (usize, usize), (usize, usize));

pub(crate) fn read_lines(
    rendered: &Rendered,
    top: usize,
    height: usize,
    theme: &Theme,
    select: Option<Select>,
) -> Vec<Line> {
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

fn rendered_line(line: &RenderedLine, theme: &Theme, sel: Option<&Range<usize>>) -> Line {
    let mut parts = Line::new();
    let mut run = String::new();
    let mut run_style = CellStyle::default();
    let mut col = 0;
    let mut started = false;
    for span in &line.spans {
        let base = theme.face(span.style).style();
        for ch in span.text.chars() {
            let mut style = base;
            if sel.is_some_and(|range| range.contains(&col)) {
                style = style.reverse();
            }
            if started && style != run_style && !run.is_empty() {
                parts.push((std::mem::take(&mut run), run_style));
            }
            run_style = style;
            started = true;
            run.push(ch);
            col += 1;
        }
    }
    if !run.is_empty() {
        parts.push((run, run_style));
    }
    parts
}

pub fn draw(frame: &mut Frame, app: &mut App, area: Rect) {
    let width = column_width(area);
    let rendered = cached_render(app, width);
    app.ui.read.read_cursor = clamp(
        &rendered,
        app.ui.read.read_cursor.0,
        app.ui.read.read_cursor.1,
    );
    let height = usize::from(area.h.max(1));
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
    for (row, line) in (inner.y..).zip(&lines) {
        put_line(&mut frame.grid, inner.x, row, line, app.ui.left, inner.w);
    }
    hscroll::markers(&mut frame.grid, inner, app.ui.left, height, |row| {
        rendered
            .lines
            .get(app.ui.read.top + row)
            .map(|line| line.text().chars().count())
    });
    let row = area.y + (app.ui.read.read_cursor.0 - app.ui.read.top) as u16;
    let col = inner.x
        + (app.ui.read.read_cursor.1.saturating_sub(app.ui.left) as u16)
            .min(inner.w.saturating_sub(1));
    frame.cursor = Some((col, row));
}
