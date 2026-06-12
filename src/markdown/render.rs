use crate::markdown::atom::Atom;
use crate::markdown::blocks::{self, Block, BlockKind};
use crate::markdown::rendered::{Rendered, RenderedLine};
use crate::markdown::style::{Span, Style};
use crate::markdown::table::TableRow;
use crate::markdown::wrap;

pub fn render(source: &str, width: usize) -> Rendered {
    let width = width.max(1);
    let mut lines = Vec::new();
    let mut prev_end: Option<usize> = None;
    for block in blocks::analyze(source) {
        if let Some(prev) = prev_end {
            for _ in 0..block.start_line.saturating_sub(prev + 1) {
                lines.push(RenderedLine::default());
            }
        }
        render_block(&block, width, &mut lines);
        prev_end = Some(block.end_line);
    }
    Rendered { lines }
}

fn render_block(block: &Block, width: usize, lines: &mut Vec<RenderedLine>) {
    match &block.kind {
        BlockKind::Heading { level, atoms } => heading(*level, atoms, width, lines),
        BlockKind::Paragraph { atoms } => {
            for line in wrap::wrap(atoms, width) {
                lines.push(RenderedLine {
                    spans: wrap::spans(&line),
                });
            }
        }
        BlockKind::Code { lang, lines: code } => code_block(lang, code, width, lines),
        BlockKind::Quote { lines: quote } => {
            for line in quote {
                for wrapped in wrap::wrap(line, width.saturating_sub(2)) {
                    let mut spans = vec![Span::decoration("\u{2502} ", Style::QuoteBar)];
                    spans.extend(wrap::spans(&wrapped));
                    lines.push(RenderedLine { spans });
                }
            }
        }
        BlockKind::Table { rows } => table(rows, lines),
        BlockKind::Verbatim { lines: raw } => {
            for line in raw {
                lines.push(RenderedLine {
                    spans: wrap::spans(line),
                });
            }
        }
    }
}

fn heading(level: u8, atoms: &[Atom], width: usize, lines: &mut Vec<RenderedLine>) {
    let title_len = atoms.len();
    match level {
        1 if width >= title_len + 4 => {
            let fill = width - title_len - 2;
            let left = fill / 2;
            let mut spans = vec![Span::decoration(
                format!("{} ", "\u{2500}".repeat(left)),
                Style::Frame,
            )];
            spans.extend(wrap::spans(atoms));
            spans.push(Span::decoration(
                format!(" {}", "\u{2500}".repeat(fill - left)),
                Style::Frame,
            ));
            lines.push(RenderedLine { spans });
        }
        2 => {
            lines.push(RenderedLine {
                spans: wrap::spans(atoms),
            });
            lines.push(RenderedLine {
                spans: vec![Span::decoration(
                    "\u{2500}".repeat(title_len.clamp(1, width)),
                    Style::Frame,
                )],
            });
        }
        _ => lines.push(RenderedLine {
            spans: wrap::spans(atoms),
        }),
    }
}

fn code_block(lang: &[Atom], code: &[Vec<Atom>], width: usize, lines: &mut Vec<RenderedLine>) {
    let width = width.max(6);
    let inner = width - 4;
    let top = if !lang.is_empty() && width >= lang.len() + 6 {
        let mut spans = vec![Span::decoration("\u{250c}\u{2500} ", Style::Frame)];
        spans.extend(wrap::spans(lang));
        spans.push(Span::decoration(
            format!(" {}\u{2510}", "\u{2500}".repeat(width - lang.len() - 5)),
            Style::Frame,
        ));
        spans
    } else {
        vec![frame_rule("\u{250c}", "\u{2510}", width)]
    };
    lines.push(RenderedLine { spans: top });
    for line in code {
        let chunks: Vec<&[Atom]> = if line.is_empty() {
            vec![&[]]
        } else {
            line.chunks(inner).collect()
        };
        for chunk in chunks {
            let mut spans = vec![Span::decoration("\u{2502} ", Style::Frame)];
            spans.extend(wrap::spans(chunk));
            spans.push(Span::decoration(
                format!("{}\u{2502}", " ".repeat(width - 3 - chunk.len())),
                Style::Frame,
            ));
            lines.push(RenderedLine { spans });
        }
    }
    lines.push(RenderedLine {
        spans: vec![frame_rule("\u{2514}", "\u{2518}", width)],
    });
}

fn frame_rule(left: &str, right: &str, width: usize) -> Span {
    Span::decoration(
        format!("{left}{}{right}", "\u{2500}".repeat(width - 2)),
        Style::Frame,
    )
}

fn table(rows: &[TableRow], lines: &mut Vec<RenderedLine>) {
    let columns = rows.iter().map(|row| row.cells.len()).max().unwrap_or(0);
    let widths: Vec<usize> = (0..columns)
        .map(|i| {
            rows.iter()
                .filter_map(|row| row.cells.get(i))
                .map(Vec::len)
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
                " ".repeat(cell_width - cell.len() + 1),
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
