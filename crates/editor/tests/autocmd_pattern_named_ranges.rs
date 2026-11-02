use v_ide_editor::autocmd::Pattern;

#[test]
fn native_pending_ranges_cross_named_classes_and_end_at_the_next_literal() {
    for source in [
        "[a--[:digit:]x]",
        "[a--[:lower:]x]",
        "[a----[:digit:]x]",
        "[a--[:digit:][:upper:]x]",
    ] {
        let compiled = Pattern::compile(source).unwrap();
        for name in ["b", "0", "-", " ", "]", "\\", "x", "\n"] {
            assert!(compiled.matches(name, name, false), "{source:?} {name:?}");
        }
        assert!(!compiled.matches("z", "z", false), "{source:?}");
    }
    for (source, preserved, excluded) in [
        ("[z--[:digit:]a]", "z", "b"),
        ("[z--[:digit:]!]", "z", "0"),
        ("[a--[:digit:]-x]", "x", "0"),
        ("[a--[:digit:]--x]", "x", "0"),
    ] {
        let compiled = Pattern::compile(source).unwrap();
        assert!(compiled.matches(preserved, preserved, false), "{source:?}");
        assert!(compiled.matches(" ", " ", false), "{source:?}");
        assert!(!compiled.matches(excluded, excluded, false), "{source:?}");
    }
    let end = Pattern::compile("[z--[:digit:]]").unwrap();
    assert!(end.matches("z", "z", false));
    assert!(end.matches("1", "1", false));
    assert!(!end.matches(" ", " ", false));
}
