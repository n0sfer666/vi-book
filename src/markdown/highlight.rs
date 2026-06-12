use std::ops::Range;

use tree_sitter::Node;

use crate::markdown::extract::extract;
use crate::markdown::parser;
use crate::markdown::style::{SourceSpan, Style};

pub fn highlight(source: &str) -> Vec<Vec<SourceSpan>> {
    let mut intervals: Vec<(Range<usize>, Style)> = Vec::new();
    if let Some(tree) = parser::parse(source) {
        collect_blocks(source, tree.block_tree().root_node(), &mut intervals);
        for inline_tree in tree.inline_trees() {
            collect_inline(source, inline_tree.root_node(), &mut intervals);
        }
    }
    let facts = extract(source);
    for link in &facts.links {
        intervals.push((link.start..link.end, Style::Link));
    }
    for tag in &facts.tags {
        intervals.push((tag.start..tag.end, Style::Tag));
    }
    if let Some(fm) = &facts.frontmatter {
        intervals.push((0..fm.end, Style::Frontmatter));
    }
    split_to_lines(source, intervals)
}

fn collect_blocks(source: &str, node: Node, out: &mut Vec<(Range<usize>, Style)>) {
    let style = match node.kind() {
        "atx_heading" => Some(heading_style(node)),
        "fenced_code_block" | "indented_code_block" => Some(Style::CodeBlock),
        _ => None,
    };
    if let Some(style) = style {
        out.push((char_range(source, node), style));
        return;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_blocks(source, child, out);
    }
}

fn heading_style(node: Node) -> Style {
    let mut cursor = node.walk();
    let level = node
        .children(&mut cursor)
        .find_map(|child| match child.kind() {
            "atx_h1_marker" => Some(1),
            "atx_h2_marker" => Some(2),
            _ => None,
        })
        .unwrap_or(3);
    match level {
        1 => Style::Heading1,
        2 => Style::Heading2,
        _ => Style::Heading3,
    }
}

fn collect_inline(source: &str, node: Node, out: &mut Vec<(Range<usize>, Style)>) {
    let style = match node.kind() {
        "strong_emphasis" => Some(Style::Bold),
        "emphasis" => Some(Style::Italic),
        "code_span" => Some(Style::CodeSpan),
        "inline_link" | "uri_autolink" => Some(Style::Link),
        _ => None,
    };
    if let Some(style) = style {
        out.push((char_range(source, node), style));
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_inline(source, child, out);
    }
}

fn char_range(source: &str, node: Node) -> Range<usize> {
    parser::byte_to_char(source, node.start_byte())..parser::byte_to_char(source, node.end_byte())
}

fn split_to_lines(source: &str, intervals: Vec<(Range<usize>, Style)>) -> Vec<Vec<SourceSpan>> {
    let mut starts = vec![0];
    for (idx, ch) in source.chars().enumerate() {
        if ch == '\n' {
            starts.push(idx + 1);
        }
    }
    let total = source.chars().count();
    let mut lines: Vec<Vec<SourceSpan>> = vec![Vec::new(); starts.len()];
    for (range, style) in intervals {
        for (line, &line_start) in starts.iter().enumerate() {
            let line_end = starts.get(line + 1).map_or(total, |&next| next - 1);
            let from = range.start.max(line_start);
            let to = range.end.min(line_end);
            if from < to {
                lines[line].push(SourceSpan {
                    start: from - line_start,
                    end: to - line_start,
                    style,
                });
            }
        }
    }
    for line in &mut lines {
        line.sort_by_key(|span| (span.start, span.end));
    }
    lines
}
