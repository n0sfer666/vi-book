mod common;

use common::{Rig, keymap, mapping};
use v_ide_keys::{Key, MapMode::Normal, Mapping, Rhs, Step, parse_keys};

#[test]
fn replaying_an_unmapped_read_retains_the_original_remapping_flag() {
    for remap in [false, true] {
        let mut map = keymap(&[(Normal, "l", "{mapped}", false)]);
        map.insert(
            Normal,
            &parse_keys("Q"),
            Mapping {
                rhs: Rhs::Sequence(vec![Rhs::Action("msg"), Rhs::Keys(parse_keys("l"))]),
                remap,
                nowait: false,
            },
        )
        .unwrap();
        let mut rig = Rig::new(map);
        rig.feed("Q");
        assert_eq!(
            rig.ta.next(&rig.map, rig.mode, false, rig.t0),
            Step::Action("msg")
        );
        assert_eq!(
            rig.ta.next(&rig.map, None, false, rig.t0),
            Step::Key(Key::char('l'))
        );
        rig.ta.replay(Key::char('l'));
        let next = rig.ta.next(&rig.map, rig.mode, false, rig.t0);
        assert_eq!(
            next,
            if remap {
                Step::Action("mapped")
            } else {
                Step::Key(Key::char('l'))
            }
        );
    }
}

#[test]
fn front_input_precedes_expansion_and_keeps_the_typed_tail_when_flushed() {
    let mut map = keymap(&[]);
    let mut rhs = mapping("l", false);
    rhs.rhs = Rhs::Sequence(vec![Rhs::Action("msg"), rhs.rhs]);
    map.insert(Normal, &parse_keys("Q"), rhs).unwrap();
    let mut rig = Rig::new(map);
    rig.feed("Qx");
    assert_eq!(
        rig.ta.next(&rig.map, rig.mode, false, rig.t0),
        Step::Action("msg")
    );
    rig.ta.feed_front(Key::char('j'));
    assert_eq!(
        rig.ta.next(&rig.map, rig.mode, false, rig.t0),
        Step::Key(Key::char('j'))
    );
    rig.ta.flush_mapped();
    assert_eq!(rig.tick(0), "x");
}
