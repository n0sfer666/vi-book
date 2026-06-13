use crate::core::edit::Edit;
use crate::core::position::Position;
use crate::vim::editor::Editor;
use crate::vim::key::Key;
use crate::vim::mode::Mode;

pub fn begin(ed: &mut Editor, pos: Position) {
    ed.mode = Mode::Insert;
    ed.cursor = pos;
    ed.insert_run.clear();
    ed.pending.reset();
}

fn start_fresh(ed: &mut Editor, pos: Position) {
    ed.insert_edits.clear();
    begin(ed, pos);
}

pub fn insert_here(ed: &mut Editor) {
    start_fresh(ed, ed.cursor);
}

pub fn append(ed: &mut Editor) {
    let col = (ed.cursor.col + 1).min(ed.buffer.line_len(ed.cursor.line));
    start_fresh(ed, Position::new(ed.cursor.line, col));
}

pub fn append_eol(ed: &mut Editor) {
    let line = ed.cursor.line;
    start_fresh(ed, Position::new(line, ed.buffer.line_len(line)));
}

pub fn insert_first_non_blank(ed: &mut Editor) {
    let line = ed.cursor.line;
    start_fresh(ed, Position::new(line, ed.buffer.first_non_blank(line)));
}

pub fn open_below(ed: &mut Editor) {
    let line = ed.cursor.line;
    let at = ed.buffer.line_start(line) + ed.buffer.line_len(line);
    start_fresh(ed, ed.cursor);
    push_edit(ed, Edit::insertion(at, "\n".into()));
    ed.cursor = Position::new(line + 1, 0);
}

pub fn open_above(ed: &mut Editor) {
    let line = ed.cursor.line;
    let at = ed.buffer.line_start(line);
    start_fresh(ed, ed.cursor);
    push_edit(ed, Edit::insertion(at, "\n".into()));
    ed.cursor = Position::new(line, 0);
}

pub fn handle(ed: &mut Editor, key: Key) {
    match key {
        Key::Esc => finalize(ed),
        Key::Char(c) => {
            insert_text(ed, c.to_string());
            ed.insert_run.push(c);
            try_escape(ed);
        }
        Key::Enter => {
            ed.insert_run.clear();
            insert_text(ed, "\n".into());
        }
        Key::Backspace => {
            ed.insert_run.clear();
            backspace(ed);
        }
        Key::Up => move_in_insert(ed, ed.cursor.line.saturating_sub(1), ed.cursor.col),
        Key::Down => move_in_insert(ed, ed.cursor.line + 1, ed.cursor.col),
        Key::Left => {
            let (line, col) = (ed.cursor.line, ed.cursor.col);
            move_in_insert(ed, line, col.saturating_sub(1));
        }
        Key::Right => move_in_insert(ed, ed.cursor.line, ed.cursor.col + 1),
        Key::Ctrl(_) => {}
    }
}

fn move_in_insert(ed: &mut Editor, line: usize, col: usize) {
    ed.insert_run.clear();
    let line = line.min(ed.buffer.line_count().saturating_sub(1));
    ed.cursor = Position::new(line, col.min(ed.buffer.line_len(line)));
    ed.sync_desired();
}

fn try_escape(ed: &mut Editor) {
    let Some(seq) = ed.insert_escape.clone() else {
        return;
    };
    if !ed.insert_run.ends_with(&seq) {
        return;
    }
    let n = seq.chars().count();
    let at = ed.cursor_index();
    if at >= n {
        let removed = ed.buffer.slice(at - n..at);
        push_edit(ed, Edit::removal(at - n, removed));
        ed.cursor = ed.buffer.char_to_pos(at - n);
    }
    finalize(ed);
}

fn insert_text(ed: &mut Editor, text: String) {
    let at = ed.cursor_index();
    let len = text.chars().count();
    push_edit(ed, Edit::insertion(at, text));
    ed.cursor = ed.buffer.char_to_pos(at + len);
}

fn backspace(ed: &mut Editor) {
    let at = ed.cursor_index();
    if at == 0 {
        return;
    }
    let removed = ed.buffer.slice(at - 1..at);
    push_edit(ed, Edit::removal(at - 1, removed));
    ed.cursor = ed.buffer.char_to_pos(at - 1);
}

fn push_edit(ed: &mut Editor, edit: Edit) {
    edit.apply(&mut ed.buffer);
    ed.insert_edits.push(edit);
}

fn finalize(ed: &mut Editor) {
    ed.mode = Mode::Normal;
    if ed.cursor.col > 0 {
        ed.cursor.col -= 1;
    }
    ed.cursor = ed.buffer.clamp_normal(ed.cursor);
    let edits = std::mem::take(&mut ed.insert_edits);
    ed.commit(edits);
    ed.sync_desired();
}
