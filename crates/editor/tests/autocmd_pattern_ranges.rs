use v_ide_editor::autocmd::{Pattern, PatternError};

#[test]
fn escaped_range_endpoints_keep_the_native_class_boundary_and_literal_suffix() {
    for (source, yes, no) in [
        (r"[]-\]]", "]]", "]"),
        (r"[--\]]", "1]", "1"),
        (r"[-\]]", "]", "]]"),
        (r"[a\-\]]", "-", "-]"),
    ] {
        let compiled = Pattern::compile(source).unwrap();
        assert!(compiled.matches(yes, yes, false), "{source:?} {yes:?}");
        assert!(!compiled.matches(no, no, false), "{source:?} {no:?}");
    }
    for source in [r"[a-\]]", r"[a-\-]", r"[]-\-]"] {
        assert_eq!(
            Pattern::compile(source).unwrap_err(),
            PatternError::ReverseRange
        );
    }
}

#[test]
fn a_named_class_after_a_hyphen_retains_the_native_outer_bracket_suffix() {
    for source in ["[[:alpha:]-[:digit:]]", "[a-[:digit:]]"] {
        let compiled = Pattern::compile(source).unwrap();
        for name in ["a]", "1]"] {
            assert!(compiled.matches(name, name, false), "{source:?} {name:?}");
        }
        assert_eq!(
            compiled.matches("-]", "-]", false),
            source.starts_with("[[:alpha:")
        );
        for name in ["a", "1", "-"] {
            assert!(!compiled.matches(name, name, false), "{source:?} {name:?}");
        }
    }
}
