use vi_book::ui::picker::Picker;
use vi_book::vault::follow::{Follow, follow, link_at};
use vi_book::vault::fuzzy::{filter, score};
use vi_book::vault::index::Index;
use vi_book::vault::note::Note;

#[test]
fn fuzzy_matches_subsequence_case_insensitively() {
    assert!(score("dn", "daily-note").is_some());
    assert!(score("xyz", "daily-note").is_none());
    assert!(score("DN", "daily-note").is_some());
}

#[test]
fn fuzzy_ranks_contiguous_and_boundary_higher() {
    let items = vec![
        "documentation".to_string(),
        "daily-note".to_string(),
        "random".to_string(),
    ];
    assert_eq!(filter("dn", &items), vec!["daily-note", "documentation"]);
}

#[test]
fn fuzzy_empty_query_keeps_order() {
    let items = vec!["b".to_string(), "a".to_string()];
    assert_eq!(filter("", &items), vec!["b", "a"]);
}

#[test]
fn link_at_finds_link_under_cursor() {
    let source = "see [[target]] here";
    assert_eq!(
        link_at(source, 7).map(|l| l.target),
        Some("target".to_string())
    );
    assert_eq!(link_at(source, 0), None);
    assert_eq!(link_at(source, 16), None);
}

#[test]
fn link_at_parses_alias_and_anchor() {
    let link = link_at("[[note#Sec|label]]", 4).unwrap();
    assert_eq!(link.target, "note");
    assert_eq!(link.anchor.as_deref(), Some("Sec"));
    assert_eq!(link.alias.as_deref(), Some("label"));
}

fn index_of(paths: &[(&str, &str)]) -> Index {
    Index::from_notes(paths.iter().map(|(p, s)| Note::from_source(p, s)).collect())
}

#[test]
fn follow_opens_unique_target() {
    let index = index_of(&[("docs/note.md", "x")]);
    let link = link_at("[[note]]", 3).unwrap();
    assert_eq!(follow(&index, &link), Follow::Open("docs/note.md".into()));
}

#[test]
fn follow_disambiguates_collisions() {
    let index = index_of(&[("a/note.md", "x"), ("b/note.md", "y")]);
    let link = link_at("[[note]]", 3).unwrap();
    assert_eq!(
        follow(&index, &link),
        Follow::Disambiguate(vec!["a/note.md".into(), "b/note.md".into()])
    );
}

#[test]
fn follow_offers_create_for_broken_link() {
    let index = index_of(&[("other.md", "x")]);
    let link = link_at("[[ghost]]", 3).unwrap();
    assert_eq!(follow(&index, &link), Follow::Create("ghost".into()));
}

#[test]
fn picker_filters_and_selects() {
    let mut picker = Picker::new(vec![
        "daily-note".to_string(),
        "documentation".to_string(),
        "random".to_string(),
    ]);
    picker.push('d');
    picker.push('n');
    assert_eq!(picker.current().as_deref(), Some("daily-note"));
}

#[test]
fn picker_selection_clamps_on_filter_change() {
    let mut picker = Picker::new(vec!["alpha".to_string(), "beta".to_string()]);
    picker.move_down();
    assert_eq!(picker.selected(), 1);
    picker.push('a');
    picker.push('l');
    assert_eq!(picker.current().as_deref(), Some("alpha"));
    assert_eq!(picker.selected(), 0);
}

#[test]
fn picker_navigation_stays_in_bounds() {
    let mut picker = Picker::new(vec!["a".to_string(), "b".to_string()]);
    picker.move_up();
    assert_eq!(picker.selected(), 0);
    picker.move_down();
    picker.move_down();
    picker.move_down();
    assert_eq!(picker.selected(), 1);
}
