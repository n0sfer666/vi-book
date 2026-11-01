mod common;

use common::{Rig, keymap, mapping};
use v_ide_keys::{MapMode::Normal, parse_keys};

fn rig(short_first: bool) -> Rig {
    let mut map = keymap(&[]);
    let mut short = mapping("{short}", false);
    short.nowait = true;
    if short_first {
        map.insert(Normal, &parse_keys("a"), short.clone()).unwrap();
    }
    map.insert(Normal, &parse_keys("ab"), mapping("{long}", false))
        .unwrap();
    if !short_first {
        map.insert(Normal, &parse_keys("a"), short).unwrap();
    }
    Rig::new(map)
}

#[test]
fn exact_nowait_fires_without_deadline_in_either_definition_order() {
    for short_first in [false, true] {
        let mut rig = rig(short_first);
        assert_eq!(rig.type_keys("a"), "{short}");
        assert!(rig.ta.deadline().is_none());
        assert_eq!(rig.type_keys("b"), "b");
    }
}

#[test]
fn already_queued_longer_match_wins() {
    for short_first in [false, true] {
        let mut rig = rig(short_first);
        rig.feed("ab");
        assert_eq!(rig.tick(0), "{long}");
    }
}

#[test]
fn nowait_ancestor_resolves_a_longer_partial_prefix() {
    let mut map = keymap(&[(Normal, "abc", "{long}", false)]);
    let mut short = mapping("{short}", false);
    short.nowait = true;
    map.insert(Normal, &parse_keys("a"), short).unwrap();
    let mut rig = Rig::new(map);
    rig.feed("ab");
    assert_eq!(rig.tick(0), "{short}b");
}

#[test]
fn clear_removes_only_the_selected_mode_including_prefixes() {
    let mut map = keymap(&[
        (Normal, "a", "{short}", false),
        (Normal, "ab", "{long}", false),
        (v_ide_keys::MapMode::Insert, "a", "{insert}", false),
    ]);
    map.clear(Normal);
    let mut rig = Rig::new(map);
    assert_eq!(rig.type_keys("ab"), "ab");
    rig.mode = Some(v_ide_keys::MapMode::Insert);
    assert_eq!(rig.type_keys("a"), "{insert}");
}
