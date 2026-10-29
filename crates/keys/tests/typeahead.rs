mod common;

use common::{Rig, keymap};
use v_ide_keys::{EmptyLhs, Keymap, MapMode::Normal};

#[test]
fn keys_without_mappings_pass_through() {
    let mut rig = Rig::new(Keymap::default());
    assert_eq!(rig.type_keys("ab<C-W><Esc>"), "ab<C-W><Esc>");
    assert_eq!(rig.pending(), "");
    assert_eq!(rig.ta.deadline(), None);
}

#[test]
fn single_key_mapping_expands_to_rhs() {
    let mut rig = Rig::new(keymap(&[(Normal, "Y", "y$", false)]));
    assert_eq!(rig.type_keys("Y"), "y$");
}

#[test]
fn action_rhs_is_returned_as_action() {
    let mut rig = Rig::new(keymap(&[(Normal, "<C-s>", "{save}", false)]));
    assert_eq!(rig.type_keys("<C-s>x"), "{save}x");
}

#[test]
fn empty_rhs_swallows_the_key() {
    let mut rig = Rig::new(keymap(&[(Normal, "Q", "", false)]));
    assert_eq!(rig.type_keys("QxQ"), "x");
}

#[test]
fn multi_key_mapping_waits_for_the_rest() {
    let mut rig = Rig::new(keymap(&[(Normal, "<Space>gh", "{hello}", false)]));
    assert_eq!(rig.type_keys("<Space>g"), "");
    assert_eq!(rig.pending(), "<Space>g");
    assert_eq!(rig.type_keys("h"), "{hello}");
    assert_eq!(rig.pending(), "");
}

#[test]
fn waiting_sets_deadline_from_timeout() {
    let mut rig = Rig::new(keymap(&[(Normal, "gh", "{hello}", false)]));
    rig.type_at("g", 5);
    assert_eq!(rig.ta.deadline(), Some(rig.at(1005)));
}

#[test]
fn broken_prefix_without_match_replays_first_key() {
    let mut rig = Rig::new(keymap(&[(Normal, "gh", "{hello}", false)]));
    assert_eq!(rig.type_keys("gx"), "gx");
    assert_eq!(rig.pending(), "");
    assert_eq!(rig.ta.deadline(), None);
}

#[test]
fn ambiguous_exact_match_waits_then_longer_wins() {
    let mut rig = Rig::new(keymap(&[
        (Normal, "a", "{a}", false),
        (Normal, "ab", "{ab}", false),
    ]));
    assert_eq!(rig.type_keys("a"), "");
    assert_eq!(rig.type_keys("b"), "{ab}");
}

#[test]
fn ambiguous_exact_match_applies_when_next_key_breaks_it() {
    let mut rig = Rig::new(keymap(&[
        (Normal, "a", "{a}", false),
        (Normal, "abc", "{abc}", false),
    ]));
    assert_eq!(rig.type_keys("abx"), "{a}bx");
}

#[test]
fn remainder_after_replayed_key_is_matched_again() {
    let mut rig = Rig::new(keymap(&[
        (Normal, "abc", "{abc}", false),
        (Normal, "bd", "{bd}", false),
    ]));
    assert_eq!(rig.type_keys("abd"), "a{bd}");
}

#[test]
fn modified_keys_are_distinct_lhs() {
    let mut rig = Rig::new(keymap(&[(Normal, "<M-x>", "{alt}", false)]));
    assert_eq!(rig.type_keys("x<M-x><C-x>"), "x{alt}<C-X>");
}

#[test]
fn insert_replaces_mapping_with_same_lhs() {
    let mut map = keymap(&[(Normal, "Q", "{old}", false)]);
    let old = map.insert(
        Normal,
        &v_ide_keys::parse_keys("Q"),
        common::mapping("{new}", false),
    );
    assert_eq!(old, Ok(Some(common::mapping("{old}", false))));
    assert_eq!(Rig::new(map).type_keys("Q"), "{new}");
}

#[test]
fn removed_mapping_passes_keys_through() {
    let mut map = keymap(&[
        (Normal, "gh", "{hello}", false),
        (Normal, "Q", "{q}", false),
    ]);
    let removed = map.remove(Normal, &v_ide_keys::parse_keys("gh"));
    assert_eq!(removed, Some(common::mapping("{hello}", false)));
    assert_eq!(map.remove(Normal, &v_ide_keys::parse_keys("gh")), None);
    assert_eq!(Rig::new(map).type_keys("ghQ"), "gh{q}");
}

#[test]
fn removing_longer_mapping_keeps_its_prefix_mapping() {
    let mut map = keymap(&[(Normal, "a", "{a}", false), (Normal, "ab", "{ab}", false)]);
    map.remove(Normal, &v_ide_keys::parse_keys("ab"));
    let mut rig = Rig::new(map);
    assert_eq!(rig.type_keys("a"), "{a}");
    assert_eq!(rig.pending(), "");
}

#[test]
fn empty_lhs_is_rejected() {
    let mut map = Keymap::default();
    let inserted = map.insert(Normal, &[], common::mapping("{x}", false));
    assert_eq!(inserted, Err(EmptyLhs));
    assert_eq!(map.remove(Normal, &[]), None);
    assert_eq!(Rig::new(map).type_keys("ab"), "ab");
}

#[test]
fn removing_shorter_mapping_keeps_longer_one() {
    let mut map = keymap(&[(Normal, "a", "{a}", false), (Normal, "ab", "{ab}", false)]);
    map.remove(Normal, &v_ide_keys::parse_keys("a"));
    let mut rig = Rig::new(map);
    assert_eq!(rig.type_keys("a"), "");
    assert_eq!(rig.pending(), "a");
    assert_eq!(rig.type_keys("b"), "{ab}");
}

#[test]
fn mode_dropped_while_waiting_replays_prefix() {
    let mut rig = Rig::new(keymap(&[(Normal, "gh", "{hello}", false)]));
    rig.type_keys("g");
    rig.mode = None;
    assert_eq!(rig.tick(0), "g");
    assert_eq!(rig.pending(), "");
}
