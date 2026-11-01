mod common;

use common::{Rig, keymap};
use v_ide_keys::{MapMode::Normal, MapRhs, Mapping, Rhs, parse_keys};

fn install(rig: &mut Rig, lhs: &str, parts: Vec<Rhs<&'static str>>, remap: bool) {
    let mapping = Mapping {
        rhs: Rhs::Sequence(parts),
        remap,
        nowait: false,
    };
    rig.map.insert(Normal, &parse_keys(lhs), mapping).unwrap();
}

#[test]
fn mixed_rhs_keeps_keys_commands_and_leader_in_order() {
    assert_eq!(
        MapRhs::parse("<Leader>j<Cmd>only<CR>k<cmd>new <Leader><cr>l", ',').unwrap(),
        MapRhs::Sequence(vec![
            MapRhs::Keys(parse_keys(",j")),
            MapRhs::Cmd("only".into()),
            MapRhs::Keys(parse_keys("k")),
            MapRhs::Cmd("new <Leader>".into()),
            MapRhs::Keys(parse_keys("l"))
        ])
    );
    for rhs in ["j<Cmd>only", "<Cmd>only<CR>k<Cmd>only"] {
        assert_eq!(
            MapRhs::parse(rhs, ' ').unwrap_err().to_string(),
            "E1255: <Cmd> mapping must end with <CR>"
        );
    }
}

#[test]
fn sequence_actions_are_emitted_between_key_segments() {
    let mut rig = Rig::new(keymap(&[]));
    install(
        &mut rig,
        "Q",
        vec![
            Rhs::Keys(parse_keys("j")),
            Rhs::Action("one"),
            Rhs::Action("two"),
            Rhs::Keys(parse_keys("k")),
        ],
        false,
    );
    rig.feed("Qx");
    assert_eq!(rig.tick(0), "j{one}{two}kx");
}

#[test]
fn remapping_stops_at_actions_and_resumes_after_them() {
    let mut rig = Rig::new(keymap(&[
        (Normal, "jk", "{wrong}", false),
        (Normal, "k", "{right}", false),
    ]));
    install(
        &mut rig,
        "Q",
        vec![
            Rhs::Keys(parse_keys("j")),
            Rhs::Action("cmd"),
            Rhs::Keys(parse_keys("k")),
        ],
        true,
    );
    assert_eq!(rig.type_keys("Q"), "j{cmd}{right}");
    assert!(rig.ta.deadline().is_none());
}

#[test]
fn flush_drops_remaining_actions_and_mapped_keys_but_keeps_typed_keys() {
    let mut rig = Rig::new(keymap(&[]));
    install(
        &mut rig,
        "Q",
        vec![
            Rhs::Action("first"),
            Rhs::Keys(parse_keys("dd")),
            Rhs::Action("last"),
        ],
        false,
    );
    rig.feed("Qj");
    let step = rig.ta.next(&rig.map, rig.mode, false, rig.t0);
    assert_eq!(step, v_ide_keys::Step::Action("first"));
    rig.ta.flush_mapped();
    assert_eq!(rig.tick(0), "j");
}

#[test]
fn nested_expansion_runs_before_outer_remainder() {
    let mut rig = Rig::new(keymap(&[]));
    install(
        &mut rig,
        "Q",
        vec![
            Rhs::Keys(parse_keys("Z")),
            Rhs::Action("outer"),
            Rhs::Keys(parse_keys("x")),
        ],
        true,
    );
    install(
        &mut rig,
        "Z",
        vec![Rhs::Action("inner"), Rhs::Keys(parse_keys("j"))],
        false,
    );
    assert_eq!(rig.type_keys("Q"), "{inner}j{outer}x");
}
