use v_ide_engine::Editor;
use v_ide_keys::MapMode::{self, Cmdline, Insert, Normal, OpPending, Visual};

fn mode_after(keys: &str) -> Option<MapMode> {
    let mut editor = Editor::new("one two\nthree four");
    editor.handle_keys(keys);
    editor.map_mode()
}

fn assert_modes(cases: &[(&str, Option<MapMode>)]) {
    for &(keys, expected) in cases {
        assert_eq!(mode_after(keys), expected, "after {keys:?}");
    }
}

#[test]
fn normal_with_or_without_count_and_register() {
    assert_modes(&[
        ("", Some(Normal)),
        ("3", Some(Normal)),
        ("12", Some(Normal)),
        ("\"a", Some(Normal)),
        ("\"a3", Some(Normal)),
        ("dd", Some(Normal)),
        ("gg", Some(Normal)),
        ("i<Esc>", Some(Normal)),
        ("v<Esc>", Some(Normal)),
    ]);
}

#[test]
fn operator_pending_after_operator_and_its_count() {
    assert_modes(&[
        ("d", Some(OpPending)),
        ("c", Some(OpPending)),
        ("y", Some(OpPending)),
        ("2d", Some(OpPending)),
        ("d3", Some(OpPending)),
        ("\"ay", Some(OpPending)),
    ]);
}

#[test]
fn character_arguments_are_not_mapped() {
    assert_modes(&[
        ("\"", None),
        ("3\"", None),
        ("g", None),
        ("dg", None),
        ("di", None),
        ("da", None),
        ("vg", None),
        ("Vg", None),
    ]);
}

#[test]
fn visual_insert_and_search() {
    assert_modes(&[
        ("v", Some(Visual)),
        ("V", Some(Visual)),
        ("<C-v>", Some(Visual)),
        ("vj", Some(Visual)),
        ("i", Some(Insert)),
        ("cw", Some(Insert)),
        ("/", Some(Cmdline)),
        ("/tw", Some(Cmdline)),
        ("/tw<Esc>", Some(Normal)),
        ("/tw<CR>", Some(Normal)),
    ]);
}

#[test]
fn counting_tracks_count_of_current_command() {
    let cases: &[(&str, bool)] = &[
        ("", false),
        ("1", true),
        ("10", true),
        ("\"a3", true),
        ("3j", false),
        ("2d", false),
        ("d2", true),
        ("2d3", true),
        ("v", false),
        ("v2", true),
        ("3\"", false),
        ("3i", false),
        ("2/", false),
    ];
    for &(keys, expected) in cases {
        let mut editor = Editor::new("one two\nthree four");
        editor.handle_keys(keys);
        assert_eq!(editor.counting(), expected, "after {keys:?}");
    }
}
