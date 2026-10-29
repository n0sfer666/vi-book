mod common;

use common::{Rig, keymap};
use v_ide_keys::MapMode::{Normal, OpPending};

#[test]
fn zero_is_mapped_outside_count() {
    let mut rig = Rig::new(keymap(&[(Normal, "0", "x", false)]));
    assert_eq!(rig.type_keys("0"), "x");
}

#[test]
fn zero_is_not_mapped_while_counting() {
    let mut rig = Rig::new(keymap(&[(Normal, "0", "x", false)])).counting();
    assert_eq!(rig.type_keys("0"), "0");
}

#[test]
fn zero_led_lhs_neither_waits_nor_matches_while_counting() {
    let mut rig = Rig::new(keymap(&[(Normal, "0j", "x", false)])).counting();
    assert_eq!(rig.type_keys("0"), "0");
    assert_eq!(rig.pending(), "");
    assert_eq!(rig.type_keys("j"), "j");
}

#[test]
fn other_keys_are_mapped_while_counting() {
    let mut rig = Rig::new(keymap(&[(Normal, "j", "gj", false)])).counting();
    assert_eq!(rig.type_keys("j"), "gj");
    let mut rig = Rig::new(keymap(&[
        (Normal, "0", "x", false),
        (Normal, "j", "gj", false),
    ]))
    .counting();
    assert_eq!(rig.type_keys("0j"), "0gj");
    let mut rig = Rig::new(keymap(&[(OpPending, "j", "k", false)]))
        .in_mode(Some(OpPending))
        .counting();
    assert_eq!(rig.type_keys("j"), "k");
}

#[test]
fn zero_inside_rhs_is_not_mapped_while_counting() {
    let mut rig = Rig::new(keymap(&[
        (Normal, "Q", "0", true),
        (Normal, "0", "x", false),
    ]))
    .counting();
    assert_eq!(rig.type_keys("Q"), "0");
}
