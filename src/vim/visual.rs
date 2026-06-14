use std::ops::Range;

use crate::core::position::Position;
use crate::vim::editor::Editor;
use crate::vim::key::Key;
use crate::vim::mode::Mode;
use crate::vim::motion::{self, Motion, OpTarget};
use crate::vim::operator::{self, Operator};
use crate::vim::pending::{Pending, Stage};

pub fn enter(ed: &mut Editor, linewise: bool) {
    ed.visual_anchor = ed.cursor;
    ed.mode = if linewise {
        Mode::VisualLine
    } else {
        Mode::Visual
    };
    ed.pending.reset();
}

pub fn enter_block(ed: &mut Editor) {
    ed.visual_anchor = ed.cursor;
    ed.mode = Mode::VisualBlock;
    ed.pending.reset();
}

pub fn handle(ed: &mut Editor, key: Key) {
    if ed.pending.stage == Stage::G {
        ed.pending.stage = Stage::Start;
        if key == Key::Char('g') {
            let count = ed.pending.count.take();
            motion::movement(ed, Motion::FirstLine, count);
        }
        return;
    }
    match key {
        Key::Esc => {
            ed.mode = Mode::Normal;
            ed.pending.reset();
        }
        Key::Ctrl('v') => ed.mode = Mode::VisualBlock,
        Key::Char(c) => visual_char(ed, c),
        _ => {
            if let Some(m) = motion::from_key(key) {
                run_motion(ed, m);
            }
        }
    }
}

fn visual_char(ed: &mut Editor, c: char) {
    match c {
        '0' if ed.pending.count.is_none() => run_motion(ed, Motion::LineStart),
        '0'..='9' => Pending::push_digit_char(&mut ed.pending.count, c),
        'g' => ed.pending.stage = Stage::G,
        'd' | 'x' => operate(ed, Operator::Delete),
        'y' => operate(ed, Operator::Yank),
        'c' => operate(ed, Operator::Change),
        'v' => ed.mode = Mode::Visual,
        'V' => ed.mode = Mode::VisualLine,
        _ => {
            if let Some(m) = motion::from_key(Key::Char(c)) {
                run_motion(ed, m);
            }
        }
    }
}

pub fn selection_cols(
    mode: Mode,
    anchor: Position,
    cursor: Position,
    line: usize,
    line_len: usize,
) -> Option<Range<usize>> {
    let top = anchor.line.min(cursor.line);
    let bottom = anchor.line.max(cursor.line);
    if line < top || line > bottom {
        return None;
    }
    let span = line_len.max(1);
    match mode {
        Mode::VisualLine => Some(0..span),
        Mode::VisualBlock => {
            let left = anchor.col.min(cursor.col);
            let right = anchor.col.max(cursor.col);
            Some(left..right + 1)
        }
        Mode::Visual => {
            let (start, end) = if (anchor.line, anchor.col) <= (cursor.line, cursor.col) {
                (anchor, cursor)
            } else {
                (cursor, anchor)
            };
            let from = if line == start.line { start.col } else { 0 };
            let to = if line == end.line { end.col + 1 } else { span };
            Some(from..to)
        }
        _ => None,
    }
}

fn run_motion(ed: &mut Editor, m: Motion) {
    let count = ed.pending.count.take();
    motion::movement(ed, m, count);
}

fn operate(ed: &mut Editor, op: Operator) {
    let target = match ed.mode {
        Mode::VisualLine => OpTarget::Lines {
            first: ed.visual_anchor.line.min(ed.cursor.line),
            last: ed.visual_anchor.line.max(ed.cursor.line),
        },
        Mode::VisualBlock => OpTarget::Block {
            first: ed.visual_anchor.line.min(ed.cursor.line),
            last: ed.visual_anchor.line.max(ed.cursor.line),
            left: ed.visual_anchor.col.min(ed.cursor.col),
            right: ed.visual_anchor.col.max(ed.cursor.col),
        },
        _ => {
            let a = ed.buffer.pos_to_char(ed.visual_anchor);
            let b = ed.cursor_index();
            OpTarget::Chars(a.min(b)..(a.max(b) + 1).min(ed.buffer.len_chars()))
        }
    };
    ed.mode = Mode::Normal;
    operator::apply(ed, op, target);
}
