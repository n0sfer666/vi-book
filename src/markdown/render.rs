use crate::markdown::atom::{split_lines, Atom};
use crate::markdown::blocks::{self, Block, BlockKind, ListItem};
use crate::markdown::render_block::{frame_rule, table};
use crate::markdown::rendered::{Rendered, RenderedLine};
use crate::markdown::style::{Span, Style};
use crate::markdown::width::{atoms_width, split_by_width};
use crate::markdown::wrap;
use crate::markdown::wrap_mode::WrapMode;

pub fn render(source: &str, width: usize) -> Rendered {
    render_with(source, width, WrapMode::All)
}

pub fn render_with(source: &str, width: usize, wrap: WrapMode) -> Rendered {
    let width = width.max(1);
    let mut lines = Vec::new();
    let mut prev_end: Option<usize> = None;
    for block in blocks::analyze(source) {
        if let Some(prev) = prev_end {
            for _ in 0..block.start_line.saturating_sub(prev + 1) {
                lines.push(RenderedLine::default());
            }
        }
        let wrap_this = wrap.wraps(&block.kind);
        render_block(&block, width, wrap_this, &mut lines);
        prev_end = Some(block.end_line);
    }
    Rendered { lines }
}

fn lines_of(atoms: &[Atom], width: usize, wrap_this: bool) -> Vec<Vec<Atom>> {
    if wrap_this {
        wrap::wrap(atoms, width)
    } else {
        split_lines(atoms)
    }
}

fn render_block(block: &Block, width: usize, wrap_this: bool, lines: &mut Vec<RenderedLine>) {
    match &block.kind {
        BlockKind::Heading { level, atoms } => heading(*level, atoms, width, wrap_this, lines),
        BlockKind::Paragraph { atoms } => push_lines(atoms, width, wrap_this, lines),
        BlockKind::Code { lang, lines: code } => code_block(lang, code, width, wrap_this, lines),
        BlockKind::List { items } => list(items, width, wrap_this, lines),
        BlockKind::Quote { lines: quote } => {
            for line in quote {
                for wrapped in lines_of(line, width.saturating_sub(2), wrap_this) {
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

fn push_lines(atoms: &[Atom], width: usize, wrap_this: bool, lines: &mut Vec<RenderedLine>) {
    for line in lines_of(atoms, width, wrap_this) {
        lines.push(RenderedLine {
            spans: wrap::spans(&line),
        });
    }
}

fn rule(width: usize) -> RenderedLine {
    RenderedLine {
        spans: vec![Span::decoration("\u{2500}".repeat(width), Style::Frame)],
    }
}

fn heading(
    level: u8,
    atoms: &[Atom],
    width: usize,
    wrap_this: bool,
    lines: &mut Vec<RenderedLine>,
) {
    let title_len = atoms_width(atoms);
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
        1 if wrap_this => {
            push_lines(atoms, width, true, lines);
            lines.push(rule(width));
        }
        1 => lines.push(RenderedLine {
            spans: wrap::spans(atoms),
        }),
        2 => {
            push_lines(atoms, width, wrap_this, lines);
            lines.push(rule(title_len.clamp(1, width)));
        }
        _ => push_lines(atoms, width, wrap_this, lines),
    }
}

fn code_block(
    lang: &[Atom],
    code: &[Vec<Atom>],
    width: usize,
    wrap_this: bool,
    lines: &mut Vec<RenderedLine>,
) {
    let avail = width.max(6);
    let lang_len = atoms_width(lang);
    let chunks: Vec<&[Atom]> = code
        .iter()
        .flat_map(|line| {
            if wrap_this {
                split_by_width(line, avail - 4)
            } else {
                vec![line.as_slice()]
            }
        })
        .collect();
    let content = chunks.iter().map(|c| atoms_width(c)).max().unwrap_or(0);
    let lang_min = if lang.is_empty() { 0 } else { lang_len + 6 };
    let width = (content + 4).max(lang_min).min(avail);
    let top = if !lang.is_empty() && width >= lang_len + 6 {
        let mut spans = vec![Span::decoration("\u{250c}\u{2500} ", Style::Frame)];
        spans.extend(wrap::spans(lang));
        spans.push(Span::decoration(
            format!(" {}\u{2510}", "\u{2500}".repeat(width - lang_len - 5)),
            Style::Frame,
        ));
        spans
    } else {
        vec![frame_rule("\u{250c}", "\u{2510}", width)]
    };
    lines.push(RenderedLine { spans: top });
    for chunk in chunks {
        let mut spans = vec![Span::decoration("\u{2502} ", Style::Frame)];
        spans.extend(wrap::spans(chunk));
        let pad = (width - 3).saturating_sub(atoms_width(chunk));
        spans.push(Span::decoration(
            format!("{}\u{2502}", " ".repeat(pad)),
            Style::Frame,
        ));
        lines.push(RenderedLine { spans });
    }
    lines.push(RenderedLine {
        spans: vec![frame_rule("\u{2514}", "\u{2518}", width)],
    });
}

fn list(items: &[ListItem], width: usize, wrap_this: bool, lines: &mut Vec<RenderedLine>) {
    for item in items {
        let indent = item.marker.chars().count();
        let body = width.saturating_sub(indent).max(1);
        let wrapped = lines_of(&item.atoms, body, wrap_this);
        if wrapped.is_empty() {
            lines.push(RenderedLine {
                spans: vec![Span::decoration(item.marker.clone(), Style::Text)],
            });
            continue;
        }
        for (i, line) in wrapped.iter().enumerate() {
            let prefix = if i == 0 {
                item.marker.clone()
            } else {
                " ".repeat(indent)
            };
            let mut spans = vec![Span::decoration(prefix, Style::Text)];
            spans.extend(wrap::spans(line));
            lines.push(RenderedLine { spans });
        }
    }
}
