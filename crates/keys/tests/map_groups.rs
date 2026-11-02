mod common;

use common::mapping;
use v_ide_keys::{
    Keymap,
    MapMode::{Cmdline, Insert, Normal, OpPending, Visual},
    parse_keys,
};

#[test]
fn common_definition_is_one_group_even_when_query_selects_one_mode() {
    let mut map = Keymap::default();
    map.insert_modes(
        &[Normal, Visual, OpPending],
        &parse_keys("Q"),
        mapping("j", true),
    )
    .unwrap();
    let groups = map.list(&[Normal], &[]);
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].modes, vec![Normal, Visual, OpPending]);
    assert_eq!(groups[0].lhs, parse_keys("Q"));
    assert_eq!(*groups[0].mapping, mapping("j", true));
}

#[test]
fn equal_independent_definitions_remain_separate_and_newest_is_first() {
    let mut map = Keymap::default();
    for mode in [Normal, Visual, OpPending] {
        map.insert(mode, &parse_keys("Q"), mapping("j", true))
            .unwrap();
    }
    let groups = map.list(&[Normal, Visual, OpPending], &[]);
    assert_eq!(
        groups.iter().map(|g| g.modes.clone()).collect::<Vec<_>>(),
        vec![vec![OpPending], vec![Visual], vec![Normal]]
    );
}

#[test]
fn redefinition_and_removal_split_the_old_group_without_losing_other_modes() {
    let mut map = Keymap::default();
    map.insert_modes(
        &[Normal, Visual, OpPending],
        &parse_keys("Q"),
        mapping("j", true),
    )
    .unwrap();
    map.insert(Normal, &parse_keys("Q"), mapping("k", false))
        .unwrap();
    let groups = map.list(&[Normal, Visual, OpPending], &[]);
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].modes, vec![Normal]);
    assert_eq!(*groups[0].mapping, mapping("k", false));
    assert_eq!(groups[1].modes, vec![Visual, OpPending]);
    map.remove(Visual, &parse_keys("Q"));
    assert_eq!(map.list(&[OpPending], &[])[0].modes, vec![OpPending]);
    map.clear(OpPending);
    assert_eq!(map.list(&[Normal, Visual, OpPending], &[]).len(), 1);
}

#[test]
fn listing_filters_lhs_prefix_and_requested_modes() {
    let mut map = Keymap::default();
    for lhs in ["zb", "za", "ab"] {
        map.insert(Normal, &parse_keys(lhs), mapping("j", false))
            .unwrap();
    }
    map.insert_modes(&[Insert, Cmdline], &parse_keys("za"), mapping("x", false))
        .unwrap();
    let groups = map.list(&[Normal], &parse_keys("z"));
    assert_eq!(
        groups.iter().map(|g| g.lhs.clone()).collect::<Vec<_>>(),
        vec![parse_keys("za"), parse_keys("zb")]
    );
    assert!(map.list(&[Normal], &parse_keys("missing")).is_empty());
    assert_eq!(
        map.list(&[Cmdline], &parse_keys("z"))[0].modes,
        vec![Insert, Cmdline]
    );
}

#[test]
fn empty_group_definition_fails_before_any_mapping_changes() {
    let mut map = Keymap::default();
    assert!(
        map.insert_modes(&[Normal, Visual], &[], mapping("j", true))
            .is_err()
    );
    assert!(map.list(&[Normal, Visual], &[]).is_empty());
}
