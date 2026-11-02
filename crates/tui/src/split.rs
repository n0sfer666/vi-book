use v_ide_view::{Grid, Rect, Style};

use crate::layout::{column_width, text_column};
use crate::read_draw::read_lines;
use crate::read_state::ReadState;
use crate::render::{Line, put_line, styled_line};
use crate::theme::Theme;
use crate::view::View;
use v_ide_editor::document::Document;
use v_ide_markdown::highlight::highlight;
use v_ide_markdown::wrap_mode::WrapMode;

pub fn split_rects(area: Rect, n: usize) -> Vec<Rect> {
    let n = n.max(1) as u16;
    if n == 1 {
        return vec![area];
    }
    let gaps = n - 1;
    let content = area.w.saturating_sub(gaps);
    let base = content / n;
    let extra = content % n;
    let mut rects = Vec::new();
    let mut x = area.x;
    for i in 0..n {
        let w = base + u16::from(i < extra);
        rects.push(Rect { x, w, ..area });
        x += w + 1;
    }
    rects
}

pub fn draw_divider(grid: &mut Grid, area: Rect, x: u16) {
    if x >= area.x + area.w {
        return;
    }
    let dim = Style::default().dim();
    for y in area.y..area.y + area.h {
        grid.put_str(x, y, "\u{2502}", dim, 1);
    }
}

pub fn draw_pane(
    grid: &mut Grid,
    theme: &Theme,
    doc: &Document,
    state: &ReadState,
    area: Rect,
    wrap: WrapMode,
) {
    draw_pane_with_numbers(grid, theme, doc, state, area, wrap, false);
}

pub(crate) fn draw_pane_with_numbers(
    grid: &mut Grid,
    theme: &Theme,
    doc: &Document,
    state: &ReadState,
    mut area: Rect,
    wrap: WrapMode,
    read_number: bool,
) {
    let height = usize::from(area.h);
    let lines: Vec<Line> = match state.view {
        View::Read => {
            let rendered =
                crate::number::render(&doc.editor.text(), column_width(area), wrap, read_number);
            area = crate::number::draw(grid, area, state.top, rendered.lines.len(), read_number);
            read_lines(&rendered, state.top, height, theme, None)
        }
        View::Edit => {
            let text = doc.editor.text();
            let highlights = highlight(&text);
            text.split('\n')
                .enumerate()
                .skip(state.top)
                .take(height)
                .map(|(idx, line)| {
                    styled_line(
                        line,
                        highlights.get(idx).map_or(&[][..], Vec::as_slice),
                        theme,
                        None,
                    )
                })
                .collect()
        }
    };
    let inner = text_column(area);
    let left = state.left;
    for (row, line) in (inner.y..).zip(&lines) {
        put_line(grid, inner.x, row, line, left, inner.w);
    }
    crate::hscroll::markers(grid, inner, left, lines.len(), |row| {
        lines
            .get(row)
            .map(|line| line.iter().map(|(text, _)| text.chars().count()).sum())
    });
}
