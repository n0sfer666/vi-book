#![allow(dead_code)]

use std::path::Path;

use v_ide_engine::Editor;
use v_ide_text::Position;
use v_ide_tui::app::App;
use v_ide_tui::requests;

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

pub fn named_doc(text: &str, name: &str) -> v_ide_editor::document::Document {
    let mut doc = v_ide_editor::document::Document::new(text);
    doc.name = name.to_string();
    doc
}

pub fn open_vault(app: &mut App, root: &Path) {
    app.workspace.set_cwd(root.to_path_buf());
    requests::startup(app);
}
