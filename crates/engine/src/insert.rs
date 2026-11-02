use crate::editor::Editor;
use crate::mode::Mode;
use v_ide_keys::{Code, Key, Mods};
use v_ide_text::edit::Edit;
use v_ide_text::position::Position;

pub fn begin(ed: &mut Editor, pos: Position) {
    ed.mode = Mode::Insert;
    ed.cursor = pos;
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
    push_edit(ed, Edit::line_insertion(at, "\n".into(), line + 1, 1));
    ed.cursor = Position::new(line + 1, 0);
}

pub fn open_above(ed: &mut Editor) {
    let line = ed.cursor.line;
    let at = ed.buffer.line_start(line);
    start_fresh(ed, ed.cursor);
    push_edit(ed, Edit::line_insertion(at, "\n".into(), line, 1));
    ed.cursor = Position::new(line, 0);
}

pub fn handle(ed: &mut Editor, key: Key) {
    let line = ed.cursor.line;
    match (key.mods(), key.code()) {
        (Mods::NONE, Code::Esc) => finalize(ed),
        (Mods::NONE, Code::Char(c)) => insert_text(ed, c.to_string()),
        (Mods::NONE, Code::Enter) => insert_text(ed, "\n".into()),
        (Mods::NONE, Code::Tab) => insert_text(ed, "\t".into()),
        (Mods::NONE, Code::Backspace) => backspace(ed),
        (Mods::NONE, Code::Delete) => delete_under(ed),
        (Mods::NONE, Code::Home) => move_in_insert(ed, line, 0),
        (Mods::NONE, Code::End) => move_in_insert(ed, line, ed.buffer.line_len(line)),
        (Mods::NONE, Code::Up) => move_in_insert(ed, line.saturating_sub(1), ed.cursor.col),
        (Mods::NONE, Code::Down) => move_in_insert(ed, line + 1, ed.cursor.col),
        (Mods::NONE, Code::Left) => move_in_insert(ed, line, ed.cursor.col.saturating_sub(1)),
        (Mods::NONE, Code::Right) => move_in_insert(ed, line, ed.cursor.col + 1),
        _ => {}
    }
}

fn move_in_insert(ed: &mut Editor, line: usize, col: usize) {
    let line = line.min(ed.buffer.line_count().saturating_sub(1));
    ed.cursor = Position::new(line, col.min(ed.buffer.line_len(line)));
    ed.sync_desired();
}

fn insert_text(ed: &mut Editor, text: String) {
    let at = ed.cursor_index();
    let len = text.chars().count();
    push_edit(ed, Edit::insertion(at, text));
    ed.cursor = ed.buffer.char_to_pos(at + len);
}

fn delete_under(ed: &mut Editor) {
    let at = ed.cursor_index();
    if at < ed.buffer.len_chars() {
        let removed = ed.buffer.slice(at..at + 1);
        push_edit(ed, Edit::removal(at, removed));
    }
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
