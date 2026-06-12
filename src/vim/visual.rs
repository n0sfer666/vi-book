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
        Key::Char(c) => visual_char(ed, c),
        _ => {}
    }
}

fn visual_char(ed: &mut Editor, c: char) {
    match c {
        '0' if ed.pending.count.is_none() => run_motion(ed, Motion::LineStart),
        '0'..='9' => {
            let digit = c as usize - '0' as usize;
            Pending::push_digit(&mut ed.pending.count, digit);
        }
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

fn run_motion(ed: &mut Editor, m: Motion) {
    let count = ed.pending.count.take();
    motion::movement(ed, m, count);
}

fn operate(ed: &mut Editor, op: Operator) {
    let target = if ed.mode == Mode::VisualLine {
        OpTarget::Lines {
            first: ed.visual_anchor.line.min(ed.cursor.line),
            last: ed.visual_anchor.line.max(ed.cursor.line),
        }
    } else {
        let a = ed.buffer.pos_to_char(ed.visual_anchor);
        let b = ed.cursor_index();
        OpTarget::Chars(a.min(b)..a.max(b) + 1)
    };
    ed.mode = Mode::Normal;
    operator::apply(ed, op, target);
}
