mod common;

use common::{check, t};

#[test]
fn counts_multiply() {
    check(&[
        t(
            "2d3w deletes six words",
            "a b c d e f g",
            (0, 0),
            "2d3w",
            "g",
            (0, 0),
        ),
        t(
            "d3j deletes four lines",
            "1\n2\n3\n4\n5",
            (0, 0),
            "d3j",
            "5",
            (0, 0),
        ),
        t(
            "3yy yanks three lines",
            "a\nb\nc\nd",
            (0, 0),
            "3yyp",
            "a\na\nb\nc\nb\nc\nd",
            (1, 0),
        ),
    ]);
}
