use crate::markdown::style::Style;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Src {
    None,
    Exact(usize),
    Block(usize, usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Atom {
    pub ch: char,
    pub style: Style,
    pub src: Src,
}

impl Atom {
    pub fn exact(ch: char, style: Style, idx: usize) -> Self {
        Self {
            ch,
            style,
            src: Src::Exact(idx),
        }
    }
}

pub fn split_lines(atoms: &[Atom]) -> Vec<Vec<Atom>> {
    let mut lines = vec![Vec::new()];
    for atom in atoms {
        if atom.ch == '\n' {
            lines.push(Vec::new());
        } else if let Some(last) = lines.last_mut() {
            last.push(*atom);
        }
    }
    lines
}

pub fn split_lines_trimmed(atoms: &[Atom]) -> Vec<Vec<Atom>> {
    let mut lines = split_lines(atoms);
    if lines.last().is_some_and(Vec::is_empty) {
        lines.pop();
    }
    lines
}
