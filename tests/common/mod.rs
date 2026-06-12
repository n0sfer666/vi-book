#![allow(dead_code)]

use vi_book::{Editor, Position};

pub struct Case {
    pub name: &'static str,
    pub before: &'static str,
    pub cursor: (usize, usize),
    pub keys: &'static str,
    pub after: &'static str,
    pub end_cursor: (usize, usize),
}

pub fn t(
    name: &'static str,
    before: &'static str,
    cursor: (usize, usize),
    keys: &'static str,
    after: &'static str,
    end_cursor: (usize, usize),
) -> Case {
    Case {
        name,
        before,
        cursor,
        keys,
        after,
        end_cursor,
    }
}

pub fn editor_at(text: &str, line: usize, col: usize) -> Editor {
    let mut editor = Editor::new(text);
    editor.set_cursor(Position::new(line, col));
    editor
}

pub fn check(cases: &[Case]) {
    let mut failures = Vec::new();
    for case in cases {
        let mut editor = editor_at(case.before, case.cursor.0, case.cursor.1);
        editor.handle_keys(case.keys);
        let text = editor.text();
        let cursor = editor.cursor();
        let want = Position::new(case.end_cursor.0, case.end_cursor.1);
        if text != case.after || cursor != want {
            failures.push(format!(
                "{}: keys {:?}\n  text got  {:?}\n  text want {:?}\n  cursor got ({}, {}), want ({}, {})",
                case.name,
                case.keys,
                text,
                case.after,
                cursor.line,
                cursor.col,
                want.line,
                want.col
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} case(s) failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
