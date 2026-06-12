use std::ops::Range;

use crate::core::edit::Edit;
use crate::core::position::Position;
use crate::vim::desired_col::DesiredCol;
use crate::vim::editor::Editor;
use crate::vim::insert;
use crate::vim::motion::OpTarget;
use crate::vim::register::RegisterContent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Delete,
    Change,
    Yank,
}

impl Operator {
    pub fn key(self) -> char {
        match self {
            Operator::Delete => 'd',
            Operator::Change => 'c',
            Operator::Yank => 'y',
        }
    }
}

pub fn apply(ed: &mut Editor, op: Operator, target: OpTarget) {
    match target {
        OpTarget::Chars(range) => chars(ed, op, range),
        OpTarget::Lines { first, last } => lines(ed, op, first, last),
    }
    ed.pending.reset();
    ed.desired = DesiredCol::Col(ed.cursor.col);
}

fn chars(ed: &mut Editor, op: Operator, range: Range<usize>) {
    if range.start >= range.end {
        return;
    }
    let text = ed.buffer.slice(range.clone());
    ed.registers
        .write(ed.pending.register, RegisterContent::charwise(text.clone()));
    match op {
        Operator::Yank => {
            let idx = ed.cursor_index().min(range.start);
            ed.cursor = ed.buffer.clamp_normal(ed.buffer.char_to_pos(idx));
        }
        Operator::Delete => {
            let edit = Edit::removal(range.start, text);
            edit.apply(&mut ed.buffer);
            ed.cursor = ed.buffer.clamp_normal(ed.buffer.char_to_pos(range.start));
            ed.commit(vec![edit]);
        }
        Operator::Change => {
            let edit = Edit::removal(range.start, text);
            edit.apply(&mut ed.buffer);
            let pos = ed.buffer.char_to_pos(range.start);
            ed.insert_edits = vec![edit];
            insert::begin(ed, pos);
        }
    }
}

fn lines(ed: &mut Editor, op: Operator, first: usize, last: usize) {
    let last = last.min(ed.buffer.line_count() - 1);
    let first = first.min(last);
    let start = ed.buffer.line_start(first);
    let content_end = ed.buffer.line_start(last) + ed.buffer.line_len(last);
    let mut reg_text = ed.buffer.slice(start..content_end);
    reg_text.push('\n');
    ed.registers
        .write(ed.pending.register, RegisterContent::linewise(reg_text));
    match op {
        Operator::Yank => {
            let line = ed.cursor.line.min(first);
            ed.cursor = ed.buffer.clamp_normal(Position::new(line, ed.cursor.col));
        }
        Operator::Delete => {
            let range = if ed.buffer.line_has_newline(last) {
                start..content_end + 1
            } else if first > 0 {
                start - 1..content_end
            } else {
                start..content_end
            };
            let edit = Edit::removal(range.start, ed.buffer.slice(range));
            edit.apply(&mut ed.buffer);
            let line = first.min(ed.buffer.line_count() - 1);
            ed.cursor = Position::new(line, ed.buffer.first_non_blank(line));
            ed.commit(vec![edit]);
        }
        Operator::Change => {
            let edit = Edit::removal(start, ed.buffer.slice(start..content_end));
            edit.apply(&mut ed.buffer);
            ed.insert_edits = vec![edit];
            insert::begin(ed, Position::new(first, 0));
        }
    }
}
