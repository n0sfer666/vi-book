use v_ide_editor::autocmd::{Pattern, PatternError, patterns};

#[test]
fn splitting_is_lazy_preserves_braces_and_escaped_commas_and_skips_empty_items() {
    let source = r",one,,{two,{three,four}},a\,b,[x,y],tail\,";
    assert_eq!(
        patterns(source).collect::<Vec<_>>(),
        ["one", "{two,{three,four}}", r"a\,b", "[x", "y]", r"tail\,"]
    );
    let mut source = patterns("valid,bad{");
    assert!(Pattern::compile(source.next().unwrap()).is_ok());
    assert_eq!(
        Pattern::compile(source.next().unwrap()).unwrap_err(),
        PatternError::MissingClose
    );
}

#[test]
fn ordinary_glob_matches_unicode_newlines_nested_groups_and_empty_alternatives() {
    for (pattern, yes, no) in [
        ("*.{md,txt}", "note.txt", "note.rs"),
        ("{a,{b,c}}?", "c中", "cxz"),
        ("{,x}y", "y", "xyy"),
        ("{}", "", "{}"),
        ("a**b", "a\nb", "ac"),
        ("?", "é", "e\u{301}"),
        ("foo.txt$", "foo.txt$", "foo.txt"),
        ("foo+bar", "foo+bar", "foobar"),
    ] {
        let compiled = Pattern::compile(pattern).unwrap();
        assert!(compiled.matches(yes, yes, false), "{pattern:?} {yes:?}");
        assert!(!compiled.matches(no, no, false), "{pattern:?} {no:?}");
    }
}

#[test]
fn patterns_without_a_slash_match_the_tail_and_slash_patterns_match_short_or_full() {
    for (pattern, short, full, expected) in [
        ("foo.txt", "dir/foo.txt", "/root/dir/foo.txt", true),
        ("a*b", "a/b", "/root/a/b", false),
        ("dir/*.txt", "dir/foo.txt", "/root/dir/foo.txt", true),
        ("*/dir/*.txt", "dir/foo.txt", "/root/dir/foo.txt", true),
        ("/other/*.txt", "dir/foo.txt", "/root/dir/foo.txt", false),
        ("foo[/]bar", "foo/bar", "/root/foo/bar", true),
        (r"\/foo", "/foo", "/foo", true),
    ] {
        assert_eq!(
            Pattern::compile(pattern)
                .unwrap()
                .matches(short, full, false),
            expected,
            "{pattern:?}"
        );
    }
}

#[test]
fn escapes_quote_glob_punctuation_and_unknown_letters_and_drop_a_trailing_backslash() {
    for (pattern, name) in [
        (r"a\*b", "a*b"),
        (r"a\?b", "a?b"),
        (r"a\[b\]", "a[b]"),
        (r"a\{b\}", "a{b}"),
        (r"a\,b", "a,b"),
        (r"a\\b", r"a\b"),
        (r"a\.b", "a.b"),
        (r"a\qb", "aqb"),
        (r"a\ b", "a b"),
        ("foo\\", "foo"),
    ] {
        assert!(
            Pattern::compile(pattern)
                .unwrap()
                .matches(name, name, false),
            "{pattern:?}"
        );
    }
}

#[test]
fn classes_use_caret_negation_ranges_initial_brackets_and_literal_bang() {
    for (pattern, yes, no) in [
        ("[a-c]", "b", "d"),
        ("[^x]", "é", "x"),
        ("[!x]", "!", "a"),
        ("[]a]", "]", "b"),
        ("[^]a]", "b", "]"),
        (r"[a\-c]", "-", "b"),
        ("[-a]", "-", "b"),
        ("[a-]", "-", "b"),
        ("a[b", "a[b", "ab"),
        (r"[a\d]", "\\", "1"),
        ("[é-中]", "١", "a"),
    ] {
        let compiled = Pattern::compile(pattern).unwrap();
        assert!(compiled.matches(yes, yes, false), "{pattern:?} {yes:?}");
        assert!(!compiled.matches(no, no, false), "{pattern:?} {no:?}");
    }
}

#[test]
fn standard_named_classes_follow_native_ascii_and_unicode_rules() {
    for (class, yes, no) in [
        ("alpha", "A", "é"),
        ("alnum", "9", "١"),
        ("digit", "1", "١"),
        ("blank", "\t", "\n"),
        ("cntrl", "\n", "a"),
        ("graph", ":", "é"),
        ("lower", "é", "É"),
        ("upper", "É", "é"),
        ("print", "中", "\n"),
        ("punct", "-", "é"),
        ("space", "\n", "a"),
        ("xdigit", "f", "g"),
    ] {
        let compiled = Pattern::compile(&format!("[[:{class}:]]")).unwrap();
        assert!(compiled.matches(yes, yes, false), "{class}");
        assert!(!compiled.matches(no, no, false), "{class}");
    }
}

#[test]
fn malformed_braces_and_reverse_ranges_are_errors() {
    for (source, error) in [
        ("a}", PatternError::MissingOpen),
        ("a{", PatternError::MissingClose),
        ("[z-a]", PatternError::ReverseRange),
    ] {
        assert_eq!(Pattern::compile(source).unwrap_err(), error);
    }
}

#[test]
fn vim_regexp_insertions_are_explicitly_rejected_until_the_regex_roadmap_steps() {
    for source in [
        r"a\d",
        r"a\(b\)",
        r"a\|b",
        r"a\+",
        r"a\=",
        r"a\@=",
        r"a\_s",
        r"a\1",
        r"a\zs",
        r"a\\\{2,3\}",
        r"[\n]",
        "[[:keyword:]]",
        "[[=a=]]",
    ] {
        assert!(
            matches!(
                Pattern::compile(source),
                Err(PatternError::UnsupportedRegex(_))
            ),
            "{source:?}"
        );
    }
}

#[test]
fn optional_case_insensitivity_applies_to_unicode_literals_ranges_and_negation() {
    for (source, name) in [
        ("É.TXT", "é.txt"),
        ("[a-c]", "B"),
        ("[^x]", "X"),
        ("[A-z]", "_"),
    ] {
        let compiled = Pattern::compile(source).unwrap();
        assert_eq!(
            compiled.matches(name, name, true),
            source != "[^x]",
            "{source:?}"
        );
    }
    assert!(!Pattern::compile("É").unwrap().matches("é", "é", false));
}

#[test]
fn deeply_nested_groups_and_overlapping_stars_do_not_recurse_or_expand_alternatives() {
    let source = format!("{}x{}", "{".repeat(10000), ",}".repeat(10000));
    let compiled = Pattern::compile(&source).unwrap();
    assert!(compiled.matches("x", "x", false));
    assert!(compiled.matches("", "", false));
    let source = format!("{}b", "*a".repeat(200));
    assert!(
        !Pattern::compile(&source)
            .unwrap()
            .matches(&"a".repeat(4000), "", false)
    );
}
