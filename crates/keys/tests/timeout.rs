mod common;

use std::time::Duration;

use common::{Rig, keymap};
use v_ide_keys::MapMode::Normal;
use v_ide_keys::TIMEOUT;

#[test]
fn timeout_is_one_second() {
    assert_eq!(TIMEOUT, Duration::from_millis(1000));
}

#[test]
fn prefix_keeps_waiting_until_deadline() {
    let mut rig = Rig::new(keymap(&[(Normal, "gh", "{hello}", false)]));
    assert_eq!(rig.type_at("g", 0), "");
    assert_eq!(rig.tick(999), "");
    assert_eq!(rig.pending(), "g");
    assert_eq!(rig.tick(1000), "g");
    assert_eq!(rig.pending(), "");
    assert_eq!(rig.ta.deadline(), None);
}

#[test]
fn timeout_applies_longest_exact_match() {
    let mut rig = Rig::new(keymap(&[
        (Normal, "a", "{a}", false),
        (Normal, "ab", "{ab}", false),
        (Normal, "abcd", "{abcd}", false),
    ]));
    assert_eq!(rig.type_at("abc", 0), "");
    assert_eq!(rig.tick(1000), "{ab}c");
}

#[test]
fn remainder_after_timeout_waits_again() {
    let mut rig = Rig::new(keymap(&[
        (Normal, "abc", "{abc}", false),
        (Normal, "bd", "{bd}", false),
    ]));
    assert_eq!(rig.type_at("ab", 0), "");
    assert_eq!(rig.tick(1000), "a");
    assert_eq!(rig.pending(), "b");
    assert_eq!(rig.ta.deadline(), Some(rig.at(2000)));
    assert_eq!(rig.tick(1999), "");
    assert_eq!(rig.tick(2000), "b");
}

#[test]
fn new_key_restarts_the_wait() {
    let mut rig = Rig::new(keymap(&[(Normal, "ghi", "{hi}", false)]));
    rig.type_at("g", 0);
    rig.type_at("h", 600);
    assert_eq!(rig.ta.deadline(), Some(rig.at(1600)));
    assert_eq!(rig.tick(1000), "");
    assert_eq!(rig.tick(1600), "gh");
}

#[test]
fn custom_timeout_is_used() {
    let mut rig = Rig::new(keymap(&[(Normal, "gh", "{hello}", false)]))
        .with_timeout(Duration::from_millis(10));
    rig.type_at("g", 0);
    assert_eq!(rig.ta.deadline(), Some(rig.at(10)));
    assert_eq!(rig.tick(10), "g");
}

#[test]
fn tick_without_pending_keys_is_idle() {
    let mut rig = Rig::new(keymap(&[(Normal, "gh", "{hello}", false)]));
    assert_eq!(rig.tick(5000), "");
    assert_eq!(rig.ta.deadline(), None);
}

#[test]
fn wait_starts_when_typeahead_is_polled() {
    let mut rig = Rig::new(keymap(&[(Normal, "gh", "{hello}", false)]));
    rig.feed("g");
    assert_eq!(rig.tick(1500), "");
    assert_eq!(rig.ta.deadline(), Some(rig.at(2500)));
}
