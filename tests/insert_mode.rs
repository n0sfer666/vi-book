mod common;

use common::{check, editor_at, t};
use vi_book::Mode;

#[test]
fn entering_insert() {
    check(&[
        t(
            "i inserts before cursor",
            "abc",
            (0, 1),
            "iX<Esc>",
            "aXbc",
            (0, 1),
        ),
        t(
            "a appends after cursor",
            "abc",
            (0, 1),
            "aX<Esc>",
            "abXc",
            (0, 2),
        ),
        t(
            "A appends at line end",
            "abc",
            (0, 0),
            "AX<Esc>",
            "abcX",
            (0, 3),
        ),
        t("A on empty line", "\nb", (0, 0), "AX<Esc>", "X\nb", (0, 0)),
        t(
            "I inserts at first non-blank",
            "  abc",
            (0, 4),
            "IX<Esc>",
            "  Xabc",
            (0, 2),
        ),
        t(
            "o opens line below",
            "a\nb",
            (0, 0),
            "oX<Esc>",
            "a\nX\nb",
            (1, 0),
        ),
        t(
            "O opens line above",
            "a\nb",
            (1, 0),
            "OX<Esc>",
            "a\nX\nb",
            (1, 0),
        ),
        t(
            "o on last line without trailing newline",
            "a",
            (0, 0),
            "oX<Esc>",
            "a\nX",
            (1, 0),
        ),
    ]);
}

#[test]
fn editing_in_insert() {
    check(&[
        t("typed chars insert", "ac", (0, 1), "ib<Esc>", "abc", (0, 1)),
        t(
            "multiple chars append",
            "ab",
            (0, 1),
            "aXY<Esc>",
            "abXY",
            (0, 3),
        ),
        t(
            "enter splits line",
            "abcd",
            (0, 2),
            "i<CR><Esc>",
            "ab\ncd",
            (1, 0),
        ),
        t(
            "enter at line end opens new line",
            "ab",
            (0, 1),
            "a<CR>x<Esc>",
            "ab\nx",
            (1, 0),
        ),
        t(
            "backspace deletes prior char",
            "abc",
            (0, 2),
            "i<BS><Esc>",
            "ac",
            (0, 0),
        ),
        t(
            "backspace joins lines",
            "ab\ncd",
            (1, 0),
            "i<BS>X<Esc>",
            "abXcd",
            (0, 2),
        ),
        t(
            "backspace at buffer start does nothing",
            "ab",
            (0, 0),
            "i<BS>X<Esc>",
            "Xab",
            (0, 0),
        ),
        t(
            "esc without typing moves cursor left",
            "abc",
            (0, 1),
            "i<Esc>",
            "abc",
            (0, 0),
        ),
        t("esc at col 0 stays", "abc", (0, 0), "i<Esc>", "abc", (0, 0)),
    ]);
}

#[test]
fn i_enters_insert_mode() {
    let mut editor = editor_at("ab", 0, 0);
    editor.handle_keys("i");
    assert_eq!(editor.mode(), Mode::Insert);
}

#[test]
fn esc_returns_to_normal_mode() {
    let mut editor = editor_at("ab", 0, 0);
    editor.handle_keys("i<Esc>");
    assert_eq!(editor.mode(), Mode::Normal);
}

#[test]
fn change_operator_enters_insert_mode() {
    let mut editor = editor_at("ab cd", 0, 0);
    editor.handle_keys("cw");
    assert_eq!(editor.mode(), Mode::Insert);
}
