use v_ide_editor::autocmd::Pattern;

#[test]
fn repeated_range_hyphens_collapse_until_an_endpoint_or_become_literal_at_the_end() {
    for source in ["[a--]", "[a---]"] {
        let compiled = Pattern::compile(source).unwrap();
        assert!(compiled.matches("a", "a", false));
        assert!(compiled.matches("-", "-", false));
        assert!(!compiled.matches("b", "b", false));
    }
    for source in ["[a--b]", "[a--z]", "[a---z]"] {
        let compiled = Pattern::compile(source).unwrap();
        assert!(compiled.matches("b", "b", false));
        assert!(!compiled.matches("-", "-", false));
    }
}

#[test]
fn literal_escapes_after_named_classes_can_consume_the_scanned_class_boundary() {
    for source in [r"[[:alpha:]-\]]", r"[[:digit:]-\]]"] {
        let compiled = Pattern::compile(source).unwrap();
        assert!(compiled.matches("]]", "]]", false));
        assert!(compiled.matches("-]", "-]", false));
        assert!(!compiled.matches(r"\]", r"\]", false));
        assert!(!compiled.matches("]", "]", false));
    }
}
