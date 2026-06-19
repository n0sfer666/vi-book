use crate::markdown::rendered::RenderedLine;
use crate::markdown::style::{Span, Style};
use crate::markdown::table::TableRow;
use crate::markdown::width::atoms_width;
use crate::markdown::wrap;

pub(crate) fn frame_rule(left: &str, right: &str, width: usize) -> Span {
    Span::decoration(
        format!(
            "{left}{}{right}",
            "\u{2500}".repeat(width.saturating_sub(2))
        ),
        Style::Frame,
    )
}

pub(crate) fn table(rows: &[TableRow], lines: &mut Vec<RenderedLine>) {
    let columns = rows.iter().map(|row| row.cells.len()).max().unwrap_or(0);
    let widths: Vec<usize> = (0..columns)
        .map(|i| {
            rows.iter()
                .filter_map(|row| row.cells.get(i))
                .map(|cell| atoms_width(cell))
                .max()
                .unwrap_or(0)
        })
        .collect();
    lines.push(table_rule(&widths, "\u{250c}", "\u{252c}", "\u{2510}"));
    for row in rows {
        if row.delimiter {
            lines.push(table_rule(&widths, "\u{251c}", "\u{253c}", "\u{2524}"));
            continue;
        }
        let mut spans = Vec::new();
        for (i, cell_width) in widths.iter().enumerate() {
            spans.push(Span::decoration("\u{2502} ", Style::Frame));
            let cell = row.cells.get(i).map_or(&[][..], Vec::as_slice);
            spans.extend(wrap::spans(cell));
            spans.push(Span::decoration(
                " ".repeat(cell_width - atoms_width(cell) + 1),
                Style::Frame,
            ));
        }
        spans.push(Span::decoration("\u{2502}", Style::Frame));
        lines.push(RenderedLine { spans });
    }
    lines.push(table_rule(&widths, "\u{2514}", "\u{2534}", "\u{2518}"));
}

fn table_rule(widths: &[usize], left: &str, mid: &str, right: &str) -> RenderedLine {
    let cols: Vec<String> = widths.iter().map(|w| "\u{2500}".repeat(w + 2)).collect();
    RenderedLine {
        spans: vec![Span::decoration(
            format!("{left}{}{right}", cols.join(mid)),
            Style::Frame,
        )],
    }
}
