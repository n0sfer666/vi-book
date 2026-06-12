use crate::core::edit::Edit;
use crate::core::position::Position;
use crate::vim::desired_col::DesiredCol;
use crate::vim::editor::Editor;
use crate::vim::register::RegisterContent;

pub fn paste(ed: &mut Editor, before: bool) {
    let count = ed.pending.count.unwrap_or(1);
    let content = ed.registers.read(ed.pending.register).cloned();
    if let Some(content) = content
        && !content.text.is_empty()
    {
        if content.linewise {
            paste_lines(ed, &content, before, count);
        } else {
            paste_chars(ed, &content, before, count);
        }
        ed.desired = DesiredCol::Col(ed.cursor.col);
    }
    ed.pending.reset();
}

fn paste_chars(ed: &mut Editor, content: &RegisterContent, before: bool, count: usize) {
    let text = content.text.repeat(count);
    let idx = ed.cursor_index();
    let at = if before || ed.buffer.line_len(ed.cursor.line) == 0 {
        idx
    } else {
        idx + 1
    };
    let len = text.chars().count();
    let edit = Edit::insertion(at, text);
    edit.apply(&mut ed.buffer);
    ed.cursor = ed.buffer.clamp_normal(ed.buffer.char_to_pos(at + len - 1));
    ed.commit(vec![edit]);
}

fn paste_lines(ed: &mut Editor, content: &RegisterContent, before: bool, count: usize) {
    let mut text = content.text.clone();
    if !text.ends_with('\n') {
        text.push('\n');
    }
    let text = text.repeat(count);
    let line = ed.cursor.line;
    let (at, text, target_line) = if before {
        (ed.buffer.line_start(line), text, line)
    } else if line + 1 < ed.buffer.line_count() || ed.buffer.line_has_newline(line) {
        (ed.buffer.line_start(line + 1), text, line + 1)
    } else {
        let stripped = text[..text.len() - 1].to_string();
        (ed.buffer.len_chars(), format!("\n{stripped}"), line + 1)
    };
    let edit = Edit::insertion(at, text);
    edit.apply(&mut ed.buffer);
    ed.cursor = Position::new(target_line, ed.buffer.first_non_blank(target_line));
    ed.commit(vec![edit]);
}
