use tree_sitter::Node;

use crate::markdown::atom::Atom;
use crate::markdown::style::Style;
use crate::markdown::{inline, parser};

pub struct TableRow {
    pub cells: Vec<Vec<Atom>>,
    pub delimiter: bool,
}

pub fn rows(source: &str, chars: &[char], node: Node) -> Vec<TableRow> {
    let mut rows = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "pipe_table_header" | "pipe_table_row" => rows.push(TableRow {
                cells: cells(source, chars, child),
                delimiter: false,
            }),
            "pipe_table_delimiter_row" => rows.push(TableRow {
                cells: Vec::new(),
                delimiter: true,
            }),
            _ => {}
        }
    }
    rows
}

fn cells(source: &str, chars: &[char], row: Node) -> Vec<Vec<Atom>> {
    let mut cells = Vec::new();
    let mut cursor = row.walk();
    for child in row.children(&mut cursor) {
        if child.kind() == "pipe_table_cell" {
            let start = parser::byte_to_char(source, child.start_byte());
            let end = parser::byte_to_char(source, child.end_byte());
            let atoms = inline::verbatim(chars, start..end, Style::Text);
            cells.push(trim(atoms));
        }
    }
    cells
}

fn trim(mut atoms: Vec<Atom>) -> Vec<Atom> {
    while atoms.first().is_some_and(|atom| atom.ch == ' ') {
        atoms.remove(0);
    }
    while atoms.last().is_some_and(|atom| atom.ch == ' ') {
        atoms.pop();
    }
    atoms
}
