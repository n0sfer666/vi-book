use crate::theme::Theme;
use v_ide_markdown::style::SourceSpan;
use v_ide_view::{Grid, Style as CellStyle};

pub(crate) type Line = Vec<(String, CellStyle)>;

pub(crate) fn put_line(grid: &mut Grid, x: u16, y: u16, line: &Line, left: usize, max_w: u16) {
    let end = x.saturating_add(max_w);
    let mut skip = left;
    let mut col = x;
    for (text, style) in line {
        let shown: String = text.chars().skip(skip).collect();
        skip = skip.saturating_sub(text.chars().count());
        col += grid.put_str(col, y, &shown, *style, end.saturating_sub(col));
    }
}

pub(crate) fn styled_line(
    line: &str,
    spans: &[SourceSpan],
    theme: &Theme,
    selection: Option<&std::ops::Range<usize>>,
) -> Line {
    let mut parts = Line::new();
    let mut run = String::new();
    let mut run_style = CellStyle::default();
    for (col, ch) in line.chars().enumerate() {
        let mut style = spans
            .iter()
            .filter(|span| span.start <= col && col < span.end)
            .min_by_key(|span| span.end - span.start)
            .map_or_else(CellStyle::default, |span| theme.face(span.style).style());
        if selection.is_some_and(|range| range.contains(&col)) {
            style = style.reverse();
        }
        if style != run_style && !run.is_empty() {
            parts.push((std::mem::take(&mut run), run_style));
        }
        run_style = style;
        run.push(ch);
    }
    if !run.is_empty() {
        parts.push((run, run_style));
    }
    parts
}
