use vi_book::notebook::history::History;

#[test]
fn back_and_forward_walk_visited_paths() {
    let mut h = History::new();
    h.visit("a");
    h.visit("b");
    h.visit("c");
    assert_eq!(h.current(), Some("c"));
    assert_eq!(h.back(), Some("b".to_string()));
    assert_eq!(h.back(), Some("a".to_string()));
    assert_eq!(h.forward(), Some("b".to_string()));
    assert_eq!(h.forward(), Some("c".to_string()));
}

#[test]
fn back_at_start_returns_none() {
    let mut h = History::new();
    h.visit("a");
    assert_eq!(h.back(), None);
    assert_eq!(h.current(), Some("a"));
}

#[test]
fn forward_at_end_returns_none() {
    let mut h = History::new();
    h.visit("a");
    h.visit("b");
    assert_eq!(h.forward(), None);
    assert_eq!(h.current(), Some("b"));
}

#[test]
fn new_visit_after_back_truncates_forward() {
    let mut h = History::new();
    h.visit("a");
    h.visit("b");
    h.visit("c");
    h.back();
    h.back();
    assert_eq!(h.current(), Some("a"));
    h.visit("d");
    assert_eq!(h.current(), Some("d"));
    assert_eq!(h.forward(), None);
    assert_eq!(h.back(), Some("a".to_string()));
}

#[test]
fn visiting_current_path_is_a_noop() {
    let mut h = History::new();
    h.visit("a");
    h.visit("a");
    assert_eq!(h.current(), Some("a"));
    assert_eq!(h.back(), None);
}

#[test]
fn empty_history_has_no_current() {
    let mut h = History::new();
    assert_eq!(h.current(), None);
    assert_eq!(h.back(), None);
    assert_eq!(h.forward(), None);
}
