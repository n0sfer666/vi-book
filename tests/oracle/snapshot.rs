use v_ide_engine::{Editor, Mode};
use v_ide_text::Position;

use crate::fixture::{Case, Snapshot};

fn mode_name(mode: Mode) -> &'static str {
    match mode {
        Mode::Normal => "normal",
        Mode::Insert => "insert",
        Mode::Visual => "visual",
        Mode::VisualLine => "visual_line",
        Mode::VisualBlock => "visual_block",
    }
}

pub fn run(case: &Case) -> Snapshot {
    let mut editor = Editor::new(&case.before);
    editor.set_cursor(Position::new(case.cursor[0], case.cursor[1]));
    editor.handle_keys(&case.keys);
    let cursor = editor.cursor();
    Snapshot {
        text: editor.text(),
        cursor: [cursor.line, cursor.col],
        mode: mode_name(editor.mode()).to_string(),
        unnamed: editor.yanked().unwrap_or_default(),
    }
}
