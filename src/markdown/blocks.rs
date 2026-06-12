use tree_sitter::Node;
use tree_sitter_md::MarkdownTree;

use crate::markdown::atom::{Atom, split_lines};
use crate::markdown::extract::extract;
use crate::markdown::facts::NoteFacts;
use crate::markdown::style::Style;
use crate::markdown::table::{self, TableRow};
use crate::markdown::{inline, parser};

pub struct Block {
    pub start_line: usize,
    pub end_line: usize,
    pub kind: BlockKind,
}

pub enum BlockKind {
    Heading {
        level: u8,
        atoms: Vec<Atom>,
    },
    Paragraph {
        atoms: Vec<Atom>,
    },
    Code {
        lang: Vec<Atom>,
        lines: Vec<Vec<Atom>>,
    },
    Quote {
        lines: Vec<Vec<Atom>>,
    },
    Table {
        rows: Vec<TableRow>,
    },
    Verbatim {
        lines: Vec<Vec<Atom>>,
    },
}

struct Ctx<'s> {
    source: &'s str,
    chars: Vec<char>,
    tree: MarkdownTree,
    facts: NoteFacts,
}

pub fn analyze(source: &str) -> Vec<Block> {
    let Some(tree) = parser::parse(source) else {
        return Vec::new();
    };
    let ctx = Ctx {
        source,
        chars: source.chars().collect(),
        tree,
        facts: extract(source),
    };
    let mut blocks = Vec::new();
    walk(&ctx, ctx.tree.block_tree().root_node(), &mut blocks);
    blocks
}

fn walk(ctx: &Ctx, node: Node, blocks: &mut Vec<Block>) {
    match node.kind() {
        "document" | "section" => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                walk(ctx, child, blocks);
            }
        }
        "minus_metadata" => blocks.push(verbatim_block(ctx, node, Style::Frontmatter)),
        "atx_heading" => blocks.push(heading(ctx, node)),
        "paragraph" => blocks.push(block(
            node,
            BlockKind::Paragraph {
                atoms: inline_atoms(ctx, node, Style::Text),
            },
        )),
        "fenced_code_block" => blocks.push(code(ctx, node)),
        "block_quote" => blocks.push(quote(ctx, node)),
        "pipe_table" => blocks.push(block(
            node,
            BlockKind::Table {
                rows: table::rows(ctx.source, &ctx.chars, node),
            },
        )),
        _ => blocks.push(verbatim_block(ctx, node, Style::Text)),
    }
}

fn block(node: Node, kind: BlockKind) -> Block {
    let start_line = node.start_position().row;
    let end = node.end_position();
    let end_line = if end.column == 0 && end.row > start_line {
        end.row - 1
    } else {
        end.row
    };
    Block {
        start_line,
        end_line,
        kind,
    }
}

fn heading(ctx: &Ctx, node: Node) -> Block {
    let level = (1..=6)
        .find(|n| child(node, &format!("atx_h{n}_marker")).is_some())
        .unwrap_or(1) as u8;
    let style = match level {
        1 => Style::Heading1,
        2 => Style::Heading2,
        _ => Style::Heading3,
    };
    block(
        node,
        BlockKind::Heading {
            level,
            atoms: inline_atoms(ctx, node, style),
        },
    )
}

fn code(ctx: &Ctx, node: Node) -> Block {
    let lang = child(node, "info_string").map_or_else(Vec::new, |info| {
        let start = parser::byte_to_char(ctx.source, info.start_byte());
        let end = parser::byte_to_char(ctx.source, info.end_byte());
        inline::verbatim(&ctx.chars, start..end, Style::Frame)
    });
    let lines = child(node, "code_fence_content").map_or_else(Vec::new, |content| {
        let start = parser::byte_to_char(ctx.source, content.start_byte());
        let end = parser::byte_to_char(ctx.source, content.end_byte());
        let atoms = inline::verbatim(&ctx.chars, start..end, Style::CodeBlock);
        let mut lines = split_lines(&atoms);
        if lines.last().is_some_and(Vec::is_empty) {
            lines.pop();
        }
        lines
    });
    block(node, BlockKind::Code { lang, lines })
}

fn quote(ctx: &Ctx, node: Node) -> Block {
    let mut lines = Vec::new();
    collect_quote_lines(ctx, node, &mut lines);
    block(node, BlockKind::Quote { lines })
}

fn collect_quote_lines(ctx: &Ctx, node: Node, lines: &mut Vec<Vec<Atom>>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "paragraph" => lines.extend(split_lines(&inline_atoms(ctx, child, Style::Quote))),
            "block_quote" => collect_quote_lines(ctx, child, lines),
            _ => {}
        }
    }
}

fn verbatim_block(ctx: &Ctx, node: Node, style: Style) -> Block {
    let start = parser::byte_to_char(ctx.source, node.start_byte());
    let end = parser::byte_to_char(ctx.source, node.end_byte());
    let atoms = inline::verbatim(&ctx.chars, start..end, style);
    let mut lines = split_lines(&atoms);
    if lines.last().is_some_and(Vec::is_empty) {
        lines.pop();
    }
    block(node, BlockKind::Verbatim { lines })
}

fn inline_atoms(ctx: &Ctx, node: Node, base: Style) -> Vec<Atom> {
    match child(node, "inline") {
        Some(content) => inline::atoms(
            ctx.source, &ctx.chars, &ctx.tree, &content, base, &ctx.facts,
        ),
        None => Vec::new(),
    }
}

fn child<'t>(node: Node<'t>, kind: &str) -> Option<Node<'t>> {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .find(|child| child.kind() == kind)
}
