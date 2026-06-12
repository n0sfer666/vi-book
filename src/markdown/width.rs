use unicode_width::UnicodeWidthChar;

use crate::markdown::atom::Atom;

pub fn char_width(c: char) -> usize {
    UnicodeWidthChar::width(c).unwrap_or(0)
}

pub fn atoms_width(atoms: &[Atom]) -> usize {
    atoms.iter().map(|atom| char_width(atom.ch)).sum()
}

pub fn split_by_width(atoms: &[Atom], width: usize) -> Vec<&[Atom]> {
    if atoms.is_empty() {
        return vec![&[]];
    }
    let width = width.max(1);
    let mut chunks = Vec::new();
    let mut start = 0;
    let mut acc = 0;
    for (i, atom) in atoms.iter().enumerate() {
        let w = char_width(atom.ch).max(1);
        if i > start && acc + w > width {
            chunks.push(&atoms[start..i]);
            start = i;
            acc = 0;
        }
        acc += w;
    }
    chunks.push(&atoms[start..]);
    chunks
}
