mod common;

use common::{check, t};

#[test]
fn inner_word() {
    check(&[
        t(
            "diw deletes inner word",
            "hello world",
            (0, 2),
            "diw",
            " world",
            (0, 0),
        ),
        t(
            "diw on last word",
            "hello world",
            (0, 8),
            "diw",
            "hello ",
            (0, 5),
        ),
        t(
            "diw on whitespace deletes spaces",
            "a   b",
            (0, 2),
            "diw",
            "ab",
            (0, 1),
        ),
        t(
            "diw on punctuation",
            "foo.bar",
            (0, 3),
            "diw",
            "foobar",
            (0, 3),
        ),
        t(
            "ciw changes word",
            "foo bar",
            (0, 4),
            "ciwbaz<Esc>",
            "foo baz",
            (0, 6),
        ),
        t(
            "yiw moves cursor to word start",
            "hello",
            (0, 2),
            "yiw",
            "hello",
            (0, 0),
        ),
    ]);
}

#[test]
fn around_word() {
    check(&[
        t(
            "daw deletes word with trailing space",
            "hello world",
            (0, 2),
            "daw",
            "world",
            (0, 0),
        ),
        t(
            "daw uses leading space when no trailing",
            "hello world",
            (0, 8),
            "daw",
            "hello",
            (0, 4),
        ),
        t(
            "caw changes word and space",
            "one two three",
            (0, 4),
            "cawX<Esc>",
            "one Xthree",
            (0, 4),
        ),
    ]);
}

#[test]
fn inner_parens() {
    check(&[
        t(
            "di( deletes inside parens",
            "f(abc)",
            (0, 3),
            "di(",
            "f()",
            (0, 2),
        ),
        t(
            "di( picks innermost pair",
            "f(a(b)c)",
            (0, 4),
            "di(",
            "f(a()c)",
            (0, 4),
        ),
        t(
            "di( from outer region",
            "f(a(b)c)",
            (0, 6),
            "di(",
            "f()",
            (0, 2),
        ),
        t(
            "di( on opening paren",
            "f(abc)",
            (0, 1),
            "di(",
            "f()",
            (0, 2),
        ),
        t(
            "di( with no parens does nothing",
            "abc",
            (0, 0),
            "di(",
            "abc",
            (0, 0),
        ),
        t(
            "ci( enters insert inside parens",
            "f(old)",
            (0, 2),
            "ci(new<Esc>",
            "f(new)",
            (0, 4),
        ),
    ]);
}

#[test]
fn inner_quotes() {
    check(&[
        t(
            "di\" deletes inside quotes",
            "say \"hi\" now",
            (0, 5),
            "di\"",
            "say \"\" now",
            (0, 5),
        ),
        t(
            "di\" before quotes searches forward",
            "say \"hi\" now",
            (0, 0),
            "di\"",
            "say \"\" now",
            (0, 5),
        ),
        t(
            "ci\" changes quoted text",
            "a \"b\" c",
            (0, 3),
            "ci\"xy<Esc>",
            "a \"xy\" c",
            (0, 4),
        ),
        t(
            "di\" with no quotes does nothing",
            "abc",
            (0, 1),
            "di\"",
            "abc",
            (0, 1),
        ),
    ]);
}

#[test]
fn inner_paragraph() {
    check(&[
        t(
            "dip deletes paragraph",
            "a\nb\n\nc",
            (0, 0),
            "dip",
            "\nc",
            (0, 0),
        ),
        t(
            "dip deletes middle paragraph",
            "a\n\nb\nc\n\nd",
            (2, 0),
            "dip",
            "a\n\n\nd",
            (2, 0),
        ),
        t(
            "dip on blank lines deletes blank run",
            "a\n\n\nb",
            (1, 0),
            "dip",
            "a\nb",
            (1, 0),
        ),
        t(
            "cip changes paragraph",
            "x\ny\n\nz",
            (0, 0),
            "cipw<Esc>",
            "w\n\nz",
            (0, 0),
        ),
    ]);
}
