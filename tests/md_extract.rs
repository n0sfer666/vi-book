use vi_book::markdown::extract::extract;
use vi_book::markdown::facts::{Tag, WikiLink};

fn link(name: &str, start: usize, end: usize) -> WikiLink {
    WikiLink {
        name: name.into(),
        start,
        end,
    }
}

fn tag(name: &str, start: usize, end: usize) -> Tag {
    Tag {
        name: name.into(),
        start,
        end,
    }
}

#[test]
fn finds_wiki_link_with_char_range() {
    let facts = extract("see [[note]] here");
    assert_eq!(facts.links, vec![link("note", 4, 12)]);
}

#[test]
fn finds_multiple_links() {
    let facts = extract("[[a]] and [[b]]");
    assert_eq!(facts.links, vec![link("a", 0, 5), link("b", 10, 15)]);
}

#[test]
fn trims_link_name() {
    let facts = extract("[[ a b ]]");
    assert_eq!(facts.links, vec![link("a b", 0, 9)]);
}

#[test]
fn ignores_links_inside_code_span() {
    assert!(extract("use `[[x]]` ok").links.is_empty());
}

#[test]
fn ignores_links_inside_fenced_block() {
    let facts = extract("```\n[[x]]\n```\n[[y]]");
    assert_eq!(facts.links, vec![link("y", 14, 19)]);
}

#[test]
fn ignores_unclosed_and_empty_links() {
    assert!(extract("a [[b").links.is_empty());
    assert!(extract("a [[]] b").links.is_empty());
}

#[test]
fn finds_tag_with_char_range() {
    let facts = extract("text #tag more");
    assert_eq!(facts.tags, vec![tag("tag", 5, 9)]);
}

#[test]
fn tag_allows_dash_and_underscore() {
    let facts = extract("#a-b_c d");
    assert_eq!(facts.tags, vec![tag("a-b_c", 0, 6)]);
}

#[test]
fn heading_marker_is_not_a_tag() {
    assert!(extract("# heading").tags.is_empty());
}

#[test]
fn hash_inside_word_is_not_a_tag() {
    assert!(extract("a#b").tags.is_empty());
}

#[test]
fn pure_numeric_tag_is_not_a_tag() {
    assert!(extract("#123").tags.is_empty());
}

#[test]
fn ignores_tags_inside_code() {
    assert!(extract("`#x`").tags.is_empty());
}

#[test]
fn parses_frontmatter_at_start() {
    let facts = extract("---\ntitle: x\n---\nbody");
    let fm = facts.frontmatter.expect("frontmatter expected");
    assert_eq!(fm.text, "title: x\n");
    assert_eq!(fm.end, 17);
}

#[test]
fn frontmatter_must_start_at_line_zero() {
    assert!(extract("x\n---\ny: 1\n---").frontmatter.is_none());
}

#[test]
fn unclosed_frontmatter_is_ignored() {
    assert!(extract("---\ntitle").frontmatter.is_none());
}

#[test]
fn frontmatter_content_is_not_scanned_for_links_and_tags() {
    let facts = extract("---\nrefs: [[x]] #y\n---\n[[z]]");
    assert_eq!(facts.links, vec![link("z", 23, 28)]);
    assert!(facts.tags.is_empty());
}

#[test]
fn empty_source_yields_no_facts() {
    let facts = extract("");
    assert!(facts.links.is_empty());
    assert!(facts.tags.is_empty());
    assert!(facts.frontmatter.is_none());
}

#[test]
fn broken_markdown_does_not_panic() {
    extract("```rust\nunclosed");
    extract("---\nbad");
    extract("**unclosed [[half");
}
