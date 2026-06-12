use insta::assert_snapshot;
use vi_book::markdown::render::render;
use vi_book::markdown::style::Style;

#[test]
fn paragraph_wraps_by_words() {
    assert_snapshot!(render("aaa bbb ccc", 7).text(), @r"
    aaa bbb
    ccc
    ");
}

#[test]
fn long_word_hard_breaks() {
    assert_snapshot!(render("hello", 3).text(), @r"
    hel
    lo
    ");
}

#[test]
fn h1_renders_centered_with_rules() {
    assert_snapshot!(render("# Title", 11).text(), @"── Title ──");
}

#[test]
fn h2_renders_with_underline() {
    assert_snapshot!(render("## Sub", 10).text(), @r"
    Sub
    ───
    ");
}

#[test]
fn h3_renders_as_plain_text_line() {
    assert_snapshot!(render("### Deep", 10).text(), @"Deep");
}

#[test]
fn fenced_code_renders_in_frame_with_language() {
    assert_snapshot!(render("```rust\nlet x;\n```", 20).text(), @r"
    ┌─ rust ───────────┐
    │ let x;           │
    └──────────────────┘
    ");
}

#[test]
fn fenced_code_without_language() {
    assert_snapshot!(render("```\ncode\n```", 10).text(), @r"
    ┌────────┐
    │ code   │
    └────────┘
    ");
}

#[test]
fn quote_renders_with_bar() {
    assert_snapshot!(render("> a\n> b", 10).text(), @r"
    │ a
    │ b
    ");
}

#[test]
fn table_renders_with_box_frame() {
    assert_snapshot!(render("| a | b |\n|---|---|\n| 1 | 2 |", 20).text(), @r"
    ┌───┬───┐
    │ a │ b │
    ├───┼───┤
    │ 1 │ 2 │
    └───┴───┘
    ");
}

#[test]
fn inline_markup_is_concealed() {
    assert_snapshot!(render("**b** *i* `c`", 20).text(), @"b i c");
}

#[test]
fn wiki_link_brackets_are_concealed() {
    assert_snapshot!(render("a [[note]] b", 20).text(), @"a note b");
}

#[test]
fn markdown_link_shows_text_only() {
    assert_snapshot!(render("see [docs](http://x) end", 30).text(), @"see docs end");
}

#[test]
fn image_renders_as_placeholder() {
    assert_snapshot!(render("![alt](img.png)", 30).text(), @"🖼 [img.png]");
}

#[test]
fn blank_lines_between_paragraphs_are_kept() {
    assert_snapshot!(render("a\n\nb", 10).text(), @r"
    a

    b
    ");
}

#[test]
fn empty_source_renders_empty() {
    assert_eq!(render("", 10).text(), "");
}

#[test]
fn broken_markdown_does_not_panic() {
    assert!(!render("```rust\nunclosed", 10).text().is_empty());
    render("**unclosed", 10);
    render("---\nbad", 10);
}

#[test]
fn h1_title_span_is_styled_and_mapped() {
    let rendered = render("# Title", 11);
    let span = rendered.lines[0]
        .spans
        .iter()
        .find(|span| span.text == "Title")
        .expect("title span");
    assert_eq!(span.style, Style::Heading1);
    assert_eq!(span.source, Some((2, 7)));
}

#[test]
fn map_roundtrips_through_wrap() {
    let rendered = render("aaa bbb", 3);
    assert_eq!(rendered.to_rendered(4), (1, 0));
    assert_eq!(rendered.to_source(1, 0), Some(4));
    assert_eq!(rendered.to_source(0, 0), Some(0));
}

#[test]
fn heading_decoration_maps_to_none() {
    let rendered = render("# Title", 11);
    assert_eq!(rendered.to_source(0, 0), None);
}

#[test]
fn link_text_roundtrips() {
    let rendered = render("x [[note]] y", 40);
    assert_eq!(rendered.to_rendered(4), (0, 2));
    assert_eq!(rendered.to_source(0, 2), Some(4));
}

#[test]
fn hidden_syntax_maps_to_nearest_rendered_position() {
    let rendered = render("x [[note]] y", 40);
    assert_eq!(rendered.to_rendered(2), (0, 1));
}

#[test]
fn positions_roundtrip_across_blocks() {
    let source = "para one\n\n## Sub\ntext here";
    let rendered = render(source, 40);
    assert_eq!(rendered.to_rendered(17), (4, 0));
    assert_eq!(rendered.to_source(4, 0), Some(17));
}
