use vi_book::vault::index::{Index, Resolution};
use vi_book::vault::link::{self, Link};
use vi_book::vault::note::Note;

fn link(target: &str, alias: Option<&str>, anchor: Option<&str>) -> Link {
    Link {
        target: target.into(),
        alias: alias.map(str::to_string),
        anchor: anchor.map(str::to_string),
    }
}

#[test]
fn parses_plain_target() {
    assert_eq!(link::parse("note"), Some(link("note", None, None)));
}

#[test]
fn parses_alias() {
    assert_eq!(
        link::parse("note|see this"),
        Some(link("note", Some("see this"), None))
    );
}

#[test]
fn parses_anchor() {
    assert_eq!(
        link::parse("note#Section"),
        Some(link("note", None, Some("Section")))
    );
}

#[test]
fn parses_anchor_and_alias_together() {
    assert_eq!(
        link::parse("note#Section|label"),
        Some(link("note", Some("label"), Some("Section")))
    );
}

#[test]
fn trims_whitespace_in_parts() {
    assert_eq!(
        link::parse(" note # Section | label "),
        Some(link("note", Some("label"), Some("Section")))
    );
}

#[test]
fn empty_target_is_rejected() {
    assert_eq!(link::parse(""), None);
    assert_eq!(link::parse("|alias"), None);
    assert_eq!(link::parse("#anchor"), None);
}

#[test]
fn empty_alias_or_anchor_becomes_none() {
    assert_eq!(link::parse("note|"), Some(link("note", None, None)));
    assert_eq!(link::parse("note#"), Some(link("note", None, None)));
}

#[test]
fn display_prefers_alias_then_target() {
    assert_eq!(link::parse("note|shown").unwrap().display(), "shown");
    assert_eq!(link::parse("note").unwrap().display(), "note");
}

fn note(path: &str, source: &str) -> Note {
    Note::from_source(path, source)
}

#[test]
fn note_extracts_name_links_tags_headings() {
    let n = note(
        "docs/a.md",
        "# Title\n\n## Sub\nlinks [[b]] and [[c|x]] #tag",
    );
    assert_eq!(n.name, "a");
    assert_eq!(n.links.len(), 2);
    assert_eq!(n.links[0].target, "b");
    assert_eq!(n.links[1].target, "c");
    assert_eq!(n.tags, vec!["tag".to_string()]);
    assert_eq!(n.headings, vec!["Title".to_string(), "Sub".to_string()]);
}

#[test]
fn resolve_returns_none_for_unknown() {
    let index = Index::from_notes(vec![note("a.md", "x")]);
    assert_eq!(index.resolve("missing"), Resolution::None);
}

#[test]
fn resolve_returns_one_for_unique_name() {
    let index = Index::from_notes(vec![note("docs/note.md", "x"), note("other.md", "y")]);
    assert_eq!(
        index.resolve("note"),
        Resolution::One("docs/note.md".into())
    );
}

#[test]
fn resolve_returns_many_sorted_for_collisions() {
    let index = Index::from_notes(vec![
        note("archive/note.md", "x"),
        note("docs/note.md", "y"),
    ]);
    assert_eq!(
        index.resolve("note"),
        Resolution::Many(vec!["archive/note.md".into(), "docs/note.md".into()])
    );
}

#[test]
fn backlinks_lists_notes_linking_to_path() {
    let index = Index::from_notes(vec![
        note("a.md", "see [[target]]"),
        note("b.md", "also [[target|x]]"),
        note("target.md", "no self ref [[target]]"),
        note("c.md", "unrelated"),
    ]);
    let mut paths: Vec<&str> = index
        .backlinks("target.md")
        .iter()
        .map(|n| n.path.as_str())
        .collect();
    paths.sort();
    assert_eq!(paths, vec!["a.md", "b.md"]);
}

#[test]
fn broken_links_reports_unresolvable_targets() {
    let index = Index::from_notes(vec![
        note("a.md", "[[exists]] and [[ghost]]"),
        note("exists.md", "ok"),
    ]);
    assert_eq!(
        index.broken_links(),
        vec![("a.md".to_string(), "ghost".to_string())]
    );
}

#[test]
fn tags_are_unique_and_sorted() {
    let index = Index::from_notes(vec![
        note("a.md", "#zeta #alpha"),
        note("b.md", "#alpha #beta"),
    ]);
    assert_eq!(
        index.tags(),
        vec!["alpha".to_string(), "beta".to_string(), "zeta".to_string()]
    );
}

#[test]
fn note_links_ignore_those_in_code() {
    let n = note("a.md", "real [[x]]\n```\n[[y]]\n```");
    assert_eq!(n.links.len(), 1);
    assert_eq!(n.links[0].target, "x");
}
