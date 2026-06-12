mod common;

use common::{check, editor_at, t};
use vi_book::Mode;

#[test]
fn charwise_visual() {
    check(&[
        t(
            "v then motion then d deletes inclusive",
            "hello",
            (0, 1),
            "vld",
            "hlo",
            (0, 1),
        ),
        t("v single char d", "abc", (0, 1), "vd", "ac", (0, 1)),
        t(
            "v with w extends to word",
            "one two three",
            (0, 0),
            "vwd",
            "wo three",
            (0, 0),
        ),
        t(
            "v with count motion",
            "abcdef",
            (0, 0),
            "v3ld",
            "ef",
            (0, 0),
        ),
        t(
            "backwards selection deletes inclusive",
            "abcde",
            (0, 3),
            "vhhd",
            "ae",
            (0, 1),
        ),
        t(
            "visual y returns cursor to selection start",
            "abcde",
            (0, 1),
            "vlly",
            "abcde",
            (0, 1),
        ),
        t("visual y then p", "ab", (0, 0), "vyp", "aab", (0, 1)),
        t(
            "visual c enters insert",
            "hello",
            (0, 0),
            "vllcXY<Esc>",
            "XYlo",
            (0, 1),
        ),
        t(
            "esc cancels visual keeping cursor",
            "abc",
            (0, 0),
            "vl<Esc>x",
            "ac",
            (0, 1),
        ),
    ]);
}

#[test]
fn linewise_visual() {
    check(&[
        t(
            "V then d deletes line",
            "a\nb\nc",
            (1, 0),
            "Vd",
            "a\nc",
            (1, 0),
        ),
        t(
            "V with j deletes two lines",
            "a\nb\nc",
            (0, 0),
            "Vjd",
            "c",
            (0, 0),
        ),
        t(
            "V y then p pastes line",
            "a\nb",
            (0, 0),
            "Vyjp",
            "a\nb\na",
            (2, 0),
        ),
        t("V G d empties buffer", "a\nb\nc", (0, 0), "VGd", "", (0, 0)),
    ]);
}

#[test]
fn v_enters_visual_mode() {
    let mut editor = editor_at("ab", 0, 0);
    editor.handle_keys("v");
    assert_eq!(editor.mode(), Mode::Visual);
}

#[test]
fn shift_v_enters_visual_line_mode() {
    let mut editor = editor_at("ab", 0, 0);
    editor.handle_keys("V");
    assert_eq!(editor.mode(), Mode::VisualLine);
}

#[test]
fn delete_returns_to_normal_mode() {
    let mut editor = editor_at("ab", 0, 0);
    editor.handle_keys("vd");
    assert_eq!(editor.mode(), Mode::Normal);
}
