use crate::markdown::atom::{Atom, Src, split_lines};
use crate::markdown::style::Span;
use crate::markdown::width::{atoms_width, split_by_width};

pub fn wrap(atoms: &[Atom], width: usize) -> Vec<Vec<Atom>> {
    let mut lines = Vec::new();
    for logical in split_lines(atoms) {
        wrap_logical(&logical, width.max(1), &mut lines);
    }
    lines
}

fn wrap_logical(atoms: &[Atom], width: usize, lines: &mut Vec<Vec<Atom>>) {
    let mut current: Vec<Atom> = Vec::new();
    for word in words(atoms) {
        match word {
            Word::Space(space) => {
                if !current.is_empty() && atoms_width(&current) < width {
                    current.push(space);
                }
            }
            Word::Text(text) => {
                if !current.is_empty() && atoms_width(&current) + atoms_width(&text) > width {
                    flush(&mut current, lines);
                }
                if atoms_width(&text) <= width {
                    current.extend_from_slice(&text);
                } else {
                    for chunk in split_by_width(&text, width) {
                        if atoms_width(&current) + atoms_width(chunk) > width {
                            flush(&mut current, lines);
                        }
                        current.extend_from_slice(chunk);
                    }
                }
            }
        }
    }
    lines.push(current);
}

fn flush(current: &mut Vec<Atom>, lines: &mut Vec<Vec<Atom>>) {
    while current.last().is_some_and(|atom| atom.ch == ' ') {
        current.pop();
    }
    lines.push(std::mem::take(current));
}

enum Word {
    Space(Atom),
    Text(Vec<Atom>),
}

fn words(atoms: &[Atom]) -> Vec<Word> {
    let mut words = Vec::new();
    let mut text: Vec<Atom> = Vec::new();
    for atom in atoms {
        if atom.ch == ' ' {
            if !text.is_empty() {
                words.push(Word::Text(std::mem::take(&mut text)));
            }
            words.push(Word::Space(*atom));
        } else {
            text.push(*atom);
        }
    }
    if !text.is_empty() {
        words.push(Word::Text(text));
    }
    words
}

pub fn spans(atoms: &[Atom]) -> Vec<Span> {
    let mut spans: Vec<Span> = Vec::new();
    for atom in atoms {
        if let Some(last) = spans.last_mut()
            && last.style == atom.style
            && continues(last, atom)
        {
            last.text.push(atom.ch);
            if let (Some((start, end)), Src::Exact(_)) = (last.source.as_mut(), atom.src) {
                *end += 1;
                let _ = start;
            }
            continue;
        }
        let source = match atom.src {
            Src::None => None,
            Src::Exact(idx) => Some((idx, idx + 1)),
            Src::Block(start, end) => Some((start, end)),
        };
        spans.push(Span {
            text: atom.ch.to_string(),
            style: atom.style,
            source,
        });
    }
    spans
}

fn continues(span: &Span, atom: &Atom) -> bool {
    match (span.source, atom.src) {
        (None, Src::None) => true,
        (Some((_, end)), Src::Exact(idx)) => {
            idx == end && span.text.chars().count() == end - source_start(span)
        }
        (Some((start, end)), Src::Block(bs, be)) => start == bs && end == be,
        _ => false,
    }
}

fn source_start(span: &Span) -> usize {
    span.source.map_or(0, |(start, _)| start)
}
