use vi_book::markdown::highlight::highlight;
use vi_book::markdown::style::{SourceSpan, Style};

fn has(spans: &[SourceSpan], start: usize, end: usize, style: Style) -> bool {
    spans
        .iter()
        .any(|span| span.start == start && span.end == end && span.style == style)
}

#[test]
fn heading_line_is_highlighted() {
    let lines = highlight("# Hi");
    assert!(has(&lines[0], 0, 4, Style::Heading1), "{:?}", lines[0]);
}

#[test]
fn h2_and_h3_levels() {
    assert!(has(&highlight("## Hi")[0], 0, 5, Style::Heading2));
    assert!(has(&highlight("### Hi")[0], 0, 6, Style::Heading3));
}

#[test]
fn bold_includes_markers() {
    let lines = highlight("a **b** c");
    assert!(has(&lines[0], 2, 7, Style::Bold), "{:?}", lines[0]);
}

#[test]
fn italic_includes_markers() {
    let lines = highlight("*i*");
    assert!(has(&lines[0], 0, 3, Style::Italic), "{:?}", lines[0]);
}

#[test]
fn code_span_includes_backticks() {
    let lines = highlight("`c`");
    assert!(has(&lines[0], 0, 3, Style::CodeSpan), "{:?}", lines[0]);
}

#[test]
fn wiki_link_is_highlighted() {
    let lines = highlight("[[note]]");
    assert!(has(&lines[0], 0, 8, Style::Link), "{:?}", lines[0]);
}

#[test]
fn tag_is_highlighted() {
    let lines = highlight("x #tag");
    assert!(has(&lines[0], 2, 6, Style::Tag), "{:?}", lines[0]);
}

#[test]
fn frontmatter_lines_are_highlighted() {
    let lines = highlight("---\na: 1\n---\nx");
    assert!(has(&lines[0], 0, 3, Style::Frontmatter), "{:?}", lines[0]);
    assert!(has(&lines[1], 0, 4, Style::Frontmatter), "{:?}", lines[1]);
    assert!(has(&lines[2], 0, 3, Style::Frontmatter), "{:?}", lines[2]);
    assert!(lines[3].iter().all(|s| s.style != Style::Frontmatter));
}

#[test]
fn fenced_code_lines_are_highlighted() {
    let lines = highlight("```\nlet x;\n```");
    assert!(has(&lines[0], 0, 3, Style::CodeBlock), "{:?}", lines[0]);
    assert!(has(&lines[1], 0, 6, Style::CodeBlock), "{:?}", lines[1]);
    assert!(has(&lines[2], 0, 3, Style::CodeBlock), "{:?}", lines[2]);
}

#[test]
fn plain_line_has_no_spans() {
    let lines = highlight("hello");
    assert!(lines[0].is_empty(), "{:?}", lines[0]);
}

#[test]
fn line_count_matches_source() {
    assert_eq!(highlight("a\nb\nc").len(), 3);
    assert_eq!(highlight("").len(), 1);
}

#[test]
fn broken_markdown_does_not_panic() {
    highlight("```rust\nunclosed");
    highlight("**unclosed");
    highlight("---\nbad");
}
