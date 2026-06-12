mod common;

use common::{check, t};

#[test]
fn undo() {
    check(&[
        t("u undoes x", "abc", (0, 0), "xu", "abc", (0, 0)),
        t("u undoes dd", "a\nb", (0, 0), "ddu", "a\nb", (0, 0)),
        t(
            "u undoes dw",
            "hello world",
            (0, 0),
            "dwu",
            "hello world",
            (0, 0),
        ),
        t(
            "u undoes insert session as one step",
            "ab",
            (0, 0),
            "iXYZ<Esc>u",
            "ab",
            (0, 0),
        ),
        t(
            "u undoes o with typed text",
            "a",
            (0, 0),
            "oxyz<Esc>u",
            "a",
            (0, 0),
        ),
        t(
            "two undos for two commands",
            "abcd",
            (0, 0),
            "xxuu",
            "abcd",
            (0, 0),
        ),
        t(
            "single undo after two x restores one",
            "abcd",
            (0, 0),
            "xxu",
            "bcd",
            (0, 0),
        ),
        t(
            "u with nothing to undo keeps state",
            "abc",
            (0, 1),
            "u",
            "abc",
            (0, 1),
        ),
        t(
            "undo restores cursor to change start",
            "hello world",
            (0, 6),
            "dwu",
            "hello world",
            (0, 6),
        ),
        t(
            "u undoes linewise paste",
            "a\nb",
            (0, 0),
            "yypu",
            "a\nb",
            (0, 0),
        ),
        t(
            "u undoes visual delete",
            "abcde",
            (0, 1),
            "vldu",
            "abcde",
            (0, 1),
        ),
        t(
            "u undoes cc to change start",
            "hello",
            (0, 2),
            "ccX<Esc>u",
            "hello",
            (0, 0),
        ),
    ]);
}

#[test]
fn redo() {
    check(&[
        t(
            "redo reapplies undone change",
            "abc",
            (0, 0),
            "xu<C-r>",
            "bc",
            (0, 0),
        ),
        t(
            "redo after multiple undos",
            "abcd",
            (0, 0),
            "xxuu<C-r>",
            "bcd",
            (0, 0),
        ),
        t(
            "new edit clears redo",
            "ab",
            (0, 0),
            "xux<C-r>",
            "b",
            (0, 0),
        ),
        t(
            "undo redo undo is stable",
            "abc",
            (0, 0),
            "dwu<C-r>u",
            "abc",
            (0, 0),
        ),
    ]);
}

#[test]
fn undo_returns_to_original() {
    check(&[t(
        "undo everything returns original buffer",
        "one two\nthree",
        (0, 0),
        "dwjddiX<Esc>uuu",
        "one two\nthree",
        (0, 0),
    )]);
}
