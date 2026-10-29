mod common;

use common::{Rig, keymap};
use v_ide_keys::MapMode::{Cmdline, Insert, Normal, OpPending, Visual};

#[test]
fn mapping_applies_only_in_its_mode() {
    let map = keymap(&[(Insert, "jk", "<Esc>", false)]);
    assert_eq!(Rig::new(map).type_keys("jk"), "jk");
    let map = keymap(&[(Insert, "jk", "<Esc>", false)]);
    assert_eq!(
        Rig::new(map).in_mode(Some(Insert)).type_keys("ajkb"),
        "a<Esc>b"
    );
}

#[test]
fn mode_is_read_for_each_key_of_rhs() {
    let mut rig = Rig::new(keymap(&[
        (Normal, "X", "ihi<Esc>", true),
        (Insert, "h", "H", false),
    ]));
    assert_eq!(rig.type_keys("X"), "iHi<Esc>");
}

#[test]
fn no_mode_disables_mappings() {
    let mut rig = Rig::new(keymap(&[(Normal, "Y", "y$", false)]));
    assert_eq!(rig.type_keys("fYY"), "fYy$");
    let mut rig = Rig::new(keymap(&[(Normal, "gh", "{hello}", false)])).in_mode(None);
    assert_eq!(rig.type_keys("g"), "g");
    assert_eq!(rig.pending(), "");
}

#[test]
fn operator_pending_uses_its_own_table() {
    let mut rig = Rig::new(keymap(&[(OpPending, "w", "iw", false)]));
    assert_eq!(rig.type_keys("dww"), "diww");
    let mut rig = Rig::new(keymap(&[(Normal, "w", "b", false)]));
    assert_eq!(rig.type_keys("dww"), "dwb");
}

#[test]
fn visual_and_cmdline_tables_are_separate() {
    let map = keymap(&[
        (Visual, "q", "{visual}", false),
        (Cmdline, "q", "{cmdline}", false),
    ]);
    let mut rig = Rig::new(map);
    assert_eq!(
        rig.type_keys("qvq<Esc>:q<CR>q"),
        "qv{visual}<Esc>:{cmdline}<CR>q"
    );
}
