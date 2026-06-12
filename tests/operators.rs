mod common;

use common::{check, t};

#[test]
fn delete_with_motions() {
    check(&[
        t(
            "dw deletes to next word",
            "hello world",
            (0, 0),
            "dw",
            "world",
            (0, 0),
        ),
        t(
            "dw mid-word deletes rest plus space",
            "hello world",
            (0, 2),
            "dw",
            "heworld",
            (0, 2),
        ),
        t(
            "dw on last word of line stops at line end",
            "hello world\nnext",
            (0, 6),
            "dw",
            "hello \nnext",
            (0, 5),
        ),
        t(
            "dw on sole word keeps empty line",
            "foo\nbar",
            (0, 0),
            "dw",
            "\nbar",
            (0, 0),
        ),
        t(
            "d2w deletes two words",
            "one two three four",
            (0, 0),
            "d2w",
            "three four",
            (0, 0),
        ),
        t(
            "2dw equals d2w",
            "one two three four",
            (0, 0),
            "2dw",
            "three four",
            (0, 0),
        ),
        t(
            "de deletes to word end",
            "hello world",
            (0, 0),
            "de",
            " world",
            (0, 0),
        ),
        t(
            "db deletes word backwards",
            "hello world",
            (0, 6),
            "db",
            "world",
            (0, 0),
        ),
        t(
            "d dollar deletes to line end",
            "hello",
            (0, 1),
            "d$",
            "h",
            (0, 0),
        ),
        t(
            "d0 deletes to line start",
            "hello",
            (0, 3),
            "d0",
            "lo",
            (0, 0),
        ),
        t(
            "dl deletes char under cursor",
            "abc",
            (0, 0),
            "dl",
            "bc",
            (0, 0),
        ),
        t("dh deletes char left", "abc", (0, 2), "dh", "ac", (0, 1)),
        t(
            "dj deletes two lines linewise",
            "a\nb\nc",
            (0, 0),
            "dj",
            "c",
            (0, 0),
        ),
        t(
            "dk deletes two lines linewise upward",
            "a\nb\nc",
            (1, 0),
            "dk",
            "c",
            (0, 0),
        ),
        t(
            "dG deletes to buffer end",
            "a\nb\nc",
            (1, 0),
            "dG",
            "a",
            (0, 0),
        ),
        t(
            "dG on single line empties buffer",
            "abc",
            (0, 0),
            "dG",
            "",
            (0, 0),
        ),
        t(
            "dgg deletes to buffer top",
            "a\nb\nc",
            (1, 0),
            "dgg",
            "c",
            (0, 0),
        ),
    ]);
}

#[test]
fn delete_lines() {
    check(&[
        t(
            "dd deletes middle line",
            "a\nb\nc",
            (1, 0),
            "dd",
            "a\nc",
            (1, 0),
        ),
        t("dd deletes first line", "a\nb", (0, 0), "dd", "b", (0, 0)),
        t(
            "dd last line without trailing newline",
            "a\nb",
            (1, 0),
            "dd",
            "a",
            (0, 0),
        ),
        t(
            "dd last line with trailing newline",
            "a\nb\n",
            (1, 0),
            "dd",
            "a\n",
            (0, 0),
        ),
        t(
            "dd only line empties buffer",
            "abc",
            (0, 0),
            "dd",
            "",
            (0, 0),
        ),
        t(
            "2dd deletes two lines",
            "a\nb\nc",
            (0, 0),
            "2dd",
            "c",
            (0, 0),
        ),
        t(
            "3dd clamps to available lines",
            "a\nb",
            (0, 0),
            "3dd",
            "",
            (0, 0),
        ),
        t(
            "dd lands on first non-blank of next line",
            "a\n  b",
            (0, 0),
            "dd",
            "  b",
            (0, 2),
        ),
    ]);
}

#[test]
fn change_with_motions() {
    check(&[
        t(
            "cw changes word like ce",
            "hello world",
            (0, 0),
            "cwbye<Esc>",
            "bye world",
            (0, 2),
        ),
        t("cw mid-word", "hello", (0, 2), "cwy<Esc>", "hey", (0, 2)),
        t(
            "ce same as cw on word",
            "hello world",
            (0, 0),
            "cebye<Esc>",
            "bye world",
            (0, 2),
        ),
        t(
            "cw on whitespace deletes to next word",
            "a  b",
            (0, 1),
            "cwX<Esc>",
            "aXb",
            (0, 1),
        ),
        t(
            "c dollar changes to line end",
            "hello",
            (0, 1),
            "c$i<Esc>",
            "hi",
            (0, 1),
        ),
        t(
            "cc replaces whole line",
            "hello\nworld",
            (0, 2),
            "ccbye<Esc>",
            "bye\nworld",
            (0, 2),
        ),
        t(
            "cc keeps surrounding lines",
            "a\nb\nc",
            (1, 0),
            "ccX<Esc>",
            "a\nX\nc",
            (1, 0),
        ),
        t(
            "cb changes word backwards",
            "hello world",
            (0, 6),
            "cbX<Esc>",
            "Xworld",
            (0, 0),
        ),
    ]);
}

#[test]
fn yank_and_paste() {
    check(&[
        t(
            "yy then p pastes line below",
            "abc\ndef",
            (0, 0),
            "yyp",
            "abc\nabc\ndef",
            (1, 0),
        ),
        t(
            "yy keeps cursor",
            "abc\ndef",
            (0, 1),
            "yy",
            "abc\ndef",
            (0, 1),
        ),
        t(
            "yw then P pastes before cursor",
            "abc def",
            (0, 0),
            "ywP",
            "abc abc def",
            (0, 3),
        ),
        t(
            "y dollar then p",
            "hello",
            (0, 1),
            "y$p",
            "heellollo",
            (0, 5),
        ),
        t(
            "yj yanks two lines",
            "a\nb\nc",
            (0, 0),
            "yjp",
            "a\na\nb\nb\nc",
            (1, 0),
        ),
        t(
            "dw then P restores text",
            "abc def",
            (0, 0),
            "dwP",
            "abc def",
            (0, 3),
        ),
        t("x then P restores char", "abc", (0, 1), "xP", "abc", (0, 1)),
        t(
            "dd then p moves line down",
            "a\nb\nc",
            (0, 0),
            "ddp",
            "b\na\nc",
            (1, 0),
        ),
        t(
            "yy then P pastes line above",
            "a\nb",
            (1, 0),
            "yyP",
            "a\nb\nb",
            (1, 0),
        ),
        t(
            "linewise p below last line without trailing newline",
            "a\nb",
            (0, 0),
            "yyGp",
            "a\nb\na",
            (2, 0),
        ),
    ]);
}

#[test]
fn delete_char() {
    check(&[
        t(
            "x deletes char under cursor",
            "abc",
            (0, 1),
            "x",
            "ac",
            (0, 1),
        ),
        t(
            "x at line end moves cursor back",
            "abc",
            (0, 2),
            "x",
            "ab",
            (0, 1),
        ),
        t(
            "3x deletes three chars",
            "abcdef",
            (0, 0),
            "3x",
            "def",
            (0, 0),
        ),
        t(
            "x on empty line does nothing",
            "a\n\nb",
            (1, 0),
            "x",
            "a\n\nb",
            (1, 0),
        ),
        t(
            "x does not join lines",
            "ab\ncd",
            (0, 1),
            "x",
            "a\ncd",
            (0, 0),
        ),
    ]);
}

#[test]
fn cancelled_operators() {
    check(&[
        t(
            "operator with invalid motion is cancelled",
            "abc",
            (0, 0),
            "dz",
            "abc",
            (0, 0),
        ),
        t(
            "esc cancels pending operator",
            "abc",
            (0, 0),
            "d<Esc>x",
            "bc",
            (0, 0),
        ),
    ]);
}
