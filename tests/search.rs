mod common;

use common::{check, t};

#[test]
fn slash_search() {
    check(&[
        t(
            "search moves to match",
            "hello world",
            (0, 0),
            "/world<CR>",
            "hello world",
            (0, 6),
        ),
        t(
            "search starts after cursor",
            "aba",
            (0, 0),
            "/a<CR>",
            "aba",
            (0, 2),
        ),
        t(
            "search wraps around",
            "abc def",
            (0, 4),
            "/abc<CR>",
            "abc def",
            (0, 0),
        ),
        t(
            "search across lines",
            "one\ntwo\nthree",
            (0, 0),
            "/three<CR>",
            "one\ntwo\nthree",
            (2, 0),
        ),
        t(
            "no match leaves cursor",
            "abc",
            (0, 0),
            "/xyz<CR>",
            "abc",
            (0, 0),
        ),
        t(
            "search is literal not regex",
            "a.c abc",
            (0, 0),
            "/a.c<CR>",
            "a.c abc",
            (0, 0),
        ),
        t(
            "esc cancels search input",
            "abc abc",
            (0, 0),
            "/abc<Esc>",
            "abc abc",
            (0, 0),
        ),
        t(
            "backspace edits search pattern",
            "axc abc",
            (0, 0),
            "/ax<BS>bc<CR>",
            "axc abc",
            (0, 4),
        ),
    ]);
}

#[test]
fn repeat_search() {
    check(&[
        t(
            "n repeats search forward",
            "ab ab ab",
            (0, 0),
            "/ab<CR>n",
            "ab ab ab",
            (0, 6),
        ),
        t(
            "n wraps around",
            "ab ab",
            (0, 0),
            "/ab<CR>nn",
            "ab ab",
            (0, 3),
        ),
        t(
            "N searches backward",
            "ab ab ab",
            (0, 0),
            "/ab<CR>nN",
            "ab ab ab",
            (0, 3),
        ),
    ]);
}
