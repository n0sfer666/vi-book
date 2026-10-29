mod common;

use common::{Map, Rig, keymap};
use v_ide_keys::MapMode::Normal;
use v_ide_keys::{Key, Mapping, Rhs};

#[test]
fn noremap_rhs_is_not_mapped_again() {
    let mut rig = Rig::new(keymap(&[
        (Normal, "a", "b", false),
        (Normal, "b", "{b}", false),
    ]));
    assert_eq!(rig.type_keys("a"), "b");
}

#[test]
fn remap_rhs_is_mapped_again() {
    let mut rig = Rig::new(keymap(&[
        (Normal, "a", "b", true),
        (Normal, "b", "c", true),
        (Normal, "c", "{c}", false),
    ]));
    assert_eq!(rig.type_keys("a"), "{c}");
}

#[test]
fn remap_rhs_can_complete_multi_key_mapping() {
    let mut rig = Rig::new(keymap(&[
        (Normal, "W", "g", true),
        (Normal, "gh", "{gh}", false),
    ]));
    assert_eq!(rig.type_keys("W"), "");
    assert_eq!(rig.pending(), "g");
    assert_eq!(rig.type_keys("h"), "{gh}");
}

#[test]
fn rhs_starting_with_lhs_skips_only_first_key() {
    let mut rig = Rig::new(keymap(&[
        (Normal, "lb", "lbz", true),
        (Normal, "b", "{b}", false),
        (Normal, "z", "{z}", false),
    ]));
    assert_eq!(rig.type_keys("lb"), "l{b}{z}");
}

#[test]
fn single_key_rhs_starting_with_lhs_does_not_loop() {
    let mut rig = Rig::new(keymap(&[
        (Normal, "x", "xzz", true),
        (Normal, "z", "{z}", false),
    ]));
    assert_eq!(rig.type_keys("x"), "x{z}{z}");
}

#[test]
fn mapping_does_not_start_at_or_cross_noremap_key() {
    let mut rig = Rig::new(keymap(&[
        (Normal, "W", "Z", false),
        (Normal, "Z", "{z}", false),
        (Normal, "Zq", "{wrong}", true),
    ]));
    assert_eq!(rig.type_keys("Wq"), "Zq");
}

fn chain(n: u32) -> Map {
    let link = |i: u32| Key::char(char::from_u32(0x4E00 + i).unwrap_or(' '));
    let mut map = Map::default();
    for i in 1..=n {
        let rhs = Rhs::Keys(vec![link(i + 1)]);
        let inserted = map.insert(Normal, &[link(i)], Mapping { rhs, remap: true });
        assert!(inserted.is_ok());
    }
    let done = Mapping {
        rhs: Rhs::Action("ok"),
        remap: false,
    };
    assert!(map.insert(Normal, &[link(n + 1)], done).is_ok());
    map
}

#[test]
fn nine_hundred_ninety_nine_expansions_are_allowed() {
    let mut rig = Rig::new(chain(998));
    assert_eq!(rig.type_keys("丁"), "{ok}");
}

#[test]
fn thousandth_expansion_is_recursive_mapping() {
    let mut rig = Rig::new(chain(999));
    assert_eq!(rig.type_keys("丁"), "!E223");
    assert_eq!(rig.pending(), "");
}

#[test]
fn recursion_drops_mapped_keys_and_keeps_typed_ones() {
    let mut rig = Rig::new(chain(999));
    rig.feed("丁ix<Esc>");
    assert_eq!(rig.tick(0), "!E223ix<Esc>");
}

#[test]
fn emitted_key_resets_recursion_depth() {
    let mut map = chain(997);
    let inserted = map.insert(
        Normal,
        &[Key::char('Q')],
        Mapping {
            rhs: Rhs::Keys(vec![Key::char('丁'), Key::char('x'), Key::char('丁')]),
            remap: true,
        },
    );
    assert!(inserted.is_ok());
    assert_eq!(Rig::new(map).type_keys("Q"), "{ok}x{ok}");
}

#[test]
fn growing_expansion_drops_mapped_keys_on_recursion() {
    let mut rig = Rig::new(keymap(&[
        (Normal, "a", "ba", true),
        (Normal, "b", "ab", true),
    ]));
    rig.feed("aix<Esc>");
    assert_eq!(rig.tick(0), "!E223ix<Esc>");
}

#[test]
fn separate_presses_of_empty_rhs_never_recurse() {
    let mut rig = Rig::new(keymap(&[(Normal, "Q", "", false)]));
    let out: String = (0..1100).map(|_| rig.type_keys("Q")).collect();
    assert_eq!(out, "");
}

#[test]
fn thousand_queued_empty_expansions_are_recursive() {
    let mut rig = Rig::new(keymap(&[(Normal, "Q", "", false)]));
    rig.feed(&"Q".repeat(999));
    assert_eq!(rig.tick(0), "");
    rig.feed(&"Q".repeat(1000));
    assert_eq!(rig.tick(0), "!E223");
}

#[test]
fn typed_key_breaks_wait_on_mapped_prefix() {
    let mut rig = Rig::new(keymap(&[
        (Normal, "W", "g", true),
        (Normal, "gh", "{gh}", false),
    ]));
    assert_eq!(rig.type_keys("Wx"), "gx");
}

#[test]
fn flush_mapped_drops_expanded_keys_and_keeps_typed() {
    let mut rig = Rig::new(keymap(&[(Normal, "a", "xyz", false)]));
    rig.feed("ab");
    let first = rig.ta.next(&rig.map, rig.mode, false, rig.t0);
    assert!(matches!(first, v_ide_keys::Step::Key(key) if key == Key::char('x')));
    rig.ta.flush_mapped();
    assert_eq!(rig.tick(0), "b");
}

#[test]
fn flush_mapped_without_expansion_keeps_typed_keys() {
    let mut rig = Rig::new(keymap(&[(Normal, "a", "xyz", false)]));
    rig.feed("cd");
    rig.ta.flush_mapped();
    assert_eq!(rig.tick(0), "cd");
}

#[test]
fn flush_mapped_clears_deadline_of_mapped_prefix() {
    let mut rig = Rig::new(keymap(&[
        (Normal, "W", "g", true),
        (Normal, "gh", "{gh}", false),
    ]));
    assert_eq!(rig.type_keys("W"), "");
    rig.ta.flush_mapped();
    assert_eq!(rig.ta.deadline(), None);
}
