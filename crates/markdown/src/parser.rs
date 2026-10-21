use std::ops::Range;

use tree_sitter::Node;
use tree_sitter_md::{MarkdownParser, MarkdownTree};

pub fn byte_to_char(source: &str, byte: usize) -> usize {
    source[..byte.min(source.len())].chars().count()
}

pub fn node_text<'s>(source: &'s str, node: &Node) -> &'s str {
    &source[node.start_byte().min(source.len())..node.end_byte().min(source.len())]
}

pub fn child_of_kind<'t>(node: Node<'t>, kind: &str) -> Option<Node<'t>> {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .find(|child| child.kind() == kind)
}

pub fn parse(source: &str) -> Option<MarkdownTree> {
    let mut parser = MarkdownParser::default();
    if source.ends_with('\n') || source.is_empty() {
        parser.parse(source.as_bytes(), None)
    } else {
        parser.parse(format!("{source}\n").as_bytes(), None)
    }
}

pub fn code_zones(source: &str) -> Vec<Range<usize>> {
    let Some(tree) = parse(source) else {
        return Vec::new();
    };
    let mut byte_ranges = Vec::new();
    collect_kinds(
        tree.block_tree().root_node(),
        &["fenced_code_block", "indented_code_block"],
        &mut byte_ranges,
    );
    for inline in tree.inline_trees() {
        collect_kinds(inline.root_node(), &["code_span"], &mut byte_ranges);
    }
    let mut bytes: Vec<usize> = byte_ranges
        .iter()
        .flat_map(|range| [range.start, range.end])
        .collect();
    bytes.sort_unstable();
    bytes.dedup();
    let chars = chars_before(source, &bytes);
    let to_char = |byte: usize| bytes.binary_search(&byte).map_or(0, |at| chars[at]);
    let mut zones: Vec<Range<usize>> = byte_ranges
        .into_iter()
        .map(|range| to_char(range.start)..to_char(range.end))
        .collect();
    zones.sort_by_key(|zone| zone.start);
    zones
}

pub fn merge_zones(zones: &[Range<usize>]) -> Vec<Range<usize>> {
    let mut merged: Vec<Range<usize>> = Vec::with_capacity(zones.len());
    for zone in zones.iter().filter(|zone| !zone.is_empty()) {
        match merged.last_mut() {
            Some(last) if zone.start <= last.end => last.end = last.end.max(zone.end),
            _ => merged.push(zone.clone()),
        }
    }
    merged
}

fn chars_before(source: &str, sorted_bytes: &[usize]) -> Vec<usize> {
    let mut starts = source.char_indices().map(|(at, _)| at).peekable();
    let mut count = 0;
    sorted_bytes
        .iter()
        .map(|&byte| {
            while starts.next_if(|&at| at < byte).is_some() {
                count += 1;
            }
            count
        })
        .collect()
}

fn collect_kinds(node: Node, kinds: &[&str], out: &mut Vec<Range<usize>>) {
    if kinds.contains(&node.kind()) {
        out.push(node.byte_range());
        return;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_kinds(child, kinds, out);
    }
}
