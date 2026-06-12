use std::ops::Range;

use tree_sitter::Node;
use tree_sitter_md::MarkdownTree;

use crate::markdown::atom::{Atom, Src};
use crate::markdown::facts::NoteFacts;
use crate::markdown::parser;
use crate::markdown::style::Style;

struct Plan {
    styles: Vec<(Range<usize>, Style)>,
    conceal: Vec<Range<usize>>,
    replace: Vec<(Range<usize>, String)>,
}

pub fn atoms(
    source: &str,
    chars: &[char],
    tree: &MarkdownTree,
    inline_node: &Node,
    base: Style,
    facts: &NoteFacts,
) -> Vec<Atom> {
    let start = parser::byte_to_char(source, inline_node.start_byte());
    let end = parser::byte_to_char(source, inline_node.end_byte());
    let mut plan = Plan {
        styles: Vec::new(),
        conceal: Vec::new(),
        replace: Vec::new(),
    };
    if let Some(inline_tree) = tree.inline_tree(inline_node) {
        walk(source, inline_tree.root_node(), &mut plan);
    }
    let mut cursor = inline_node.walk();
    for child in inline_node.children(&mut cursor) {
        if child.kind() == "block_continuation" {
            let from = parser::byte_to_char(source, child.start_byte());
            let to = parser::byte_to_char(source, child.end_byte());
            plan.conceal.push(from..to);
        }
    }
    for link in &facts.links {
        if link.start >= start && link.end <= end {
            plan.conceal.push(link.start..link.start + 2);
            plan.conceal.push(link.end - 2..link.end);
            plan.styles
                .push((link.start + 2..link.end - 2, Style::Link));
        }
    }
    for tag in &facts.tags {
        if tag.start >= start && tag.end <= end {
            plan.styles.push((tag.start..tag.end, Style::Tag));
        }
    }
    emit(chars, start..end, base, &plan)
}

pub fn verbatim(chars: &[char], range: Range<usize>, style: Style) -> Vec<Atom> {
    range
        .filter_map(|idx| chars.get(idx).map(|&ch| Atom::exact(ch, style, idx)))
        .collect()
}

fn walk(source: &str, node: Node, plan: &mut Plan) {
    let range = || {
        parser::byte_to_char(source, node.start_byte())
            ..parser::byte_to_char(source, node.end_byte())
    };
    match node.kind() {
        "strong_emphasis" => plan.styles.push((range(), Style::Bold)),
        "emphasis" => plan.styles.push((range(), Style::Italic)),
        "code_span" => plan.styles.push((range(), Style::CodeSpan)),
        "emphasis_delimiter" | "code_span_delimiter" | "block_continuation" => {
            plan.conceal.push(range());
            return;
        }
        "inline_link" => {
            link(source, node, plan);
            return;
        }
        "image" => {
            image(source, node, plan);
            return;
        }
        _ => {}
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk(source, child, plan);
    }
}

fn link(source: &str, node: Node, plan: &mut Plan) {
    let outer = parser::byte_to_char(source, node.start_byte())
        ..parser::byte_to_char(source, node.end_byte());
    let Some(text) = child_of_kind(node, "link_text") else {
        return;
    };
    let inner = parser::byte_to_char(source, text.start_byte())
        ..parser::byte_to_char(source, text.end_byte());
    plan.styles.push((inner.clone(), Style::Link));
    plan.conceal.push(outer.start..inner.start);
    plan.conceal.push(inner.end..outer.end);
}

fn image(source: &str, node: Node, plan: &mut Plan) {
    let range = parser::byte_to_char(source, node.start_byte())
        ..parser::byte_to_char(source, node.end_byte());
    let name = child_of_kind(node, "link_destination")
        .map(|dest| parser::node_text(source, &dest).to_string())
        .unwrap_or_default();
    plan.replace.push((range, format!("\u{1f5bc} [{name}]")));
}

fn child_of_kind<'t>(node: Node<'t>, kind: &str) -> Option<Node<'t>> {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .find(|child| child.kind() == kind)
}

fn emit(chars: &[char], range: Range<usize>, base: Style, plan: &Plan) -> Vec<Atom> {
    let mut atoms = Vec::new();
    let mut i = range.start;
    while i < range.end.min(chars.len()) {
        if let Some((replace_range, text)) = plan.replace.iter().find(|(r, _)| r.start == i) {
            for ch in text.chars() {
                atoms.push(Atom {
                    ch,
                    style: Style::ImagePlaceholder,
                    src: Src::Block(replace_range.start, replace_range.end),
                });
            }
            i = replace_range.end;
            continue;
        }
        if plan.conceal.iter().any(|r| r.contains(&i)) {
            i += 1;
            continue;
        }
        atoms.push(Atom::exact(chars[i], style_at(plan, i, base), i));
        i += 1;
    }
    atoms
}

fn style_at(plan: &Plan, idx: usize, base: Style) -> Style {
    plan.styles
        .iter()
        .filter(|(range, _)| range.contains(&idx))
        .min_by_key(|(range, _)| range.end - range.start)
        .map_or(base, |(_, style)| *style)
}
