mod common;

use common::{check, t};

#[test]
fn named_registers() {
    check(&[
        t(
            "named register yank and paste",
            "hello world",
            (0, 0),
            "\"ayw$\"ap",
            "hello worldhello ",
            (0, 16),
        ),
        t(
            "named registers are independent",
            "one two three",
            (0, 0),
            "\"ayww\"byw0\"bP",
            "two one two three",
            (0, 3),
        ),
        t(
            "named register survives unnamed overwrite",
            "ab cd",
            (0, 0),
            "\"aywx$\"ap",
            "b cdab ",
            (0, 6),
        ),
        t(
            "named register with dd",
            "a\nb\nc",
            (0, 0),
            "\"addG\"ap",
            "b\nc\na",
            (2, 0),
        ),
        t(
            "yank into named register also sets unnamed",
            "ab",
            (0, 0),
            "\"aywp",
            "aabb",
            (0, 2),
        ),
    ]);
}

#[test]
fn unnamed_register() {
    check(&[
        t(
            "x writes unnamed register",
            "ab",
            (0, 0),
            "x$p",
            "ba",
            (0, 1),
        ),
        t(
            "c writes unnamed register",
            "hello world",
            (0, 0),
            "cwX<Esc>$p",
            "X worldhello",
            (0, 11),
        ),
    ]);
}
