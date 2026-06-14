use crate::vim::editor::Editor;
use crate::vim::key::Key;
use crate::vim::motion::{self, Motion, OpTarget};
use crate::vim::operator::{self, Operator};
use crate::vim::paste;
use crate::vim::pending::{Pending, Stage};
use crate::vim::search;
use crate::vim::text_object;
use crate::vim::{insert, visual};

pub fn handle(ed: &mut Editor, key: Key) {
    match ed.pending.stage {
        Stage::Start => start(ed, key),
        Stage::Register => register(ed, key),
        Stage::G => g_prefix(ed, key),
        Stage::Operator(op) => operator_stage(ed, op, key),
        Stage::OperatorG(op) => operator_g(ed, op, key),
        Stage::OperatorObject(op, around) => object_stage(ed, op, around, key),
    }
}

fn start(ed: &mut Editor, key: Key) {
    match key {
        Key::Char(c) => start_char(ed, c),
        Key::Ctrl('r') => {
            ed.redo();
            ed.pending.reset();
        }
        Key::Ctrl('v') => visual::enter_block(ed),
        _ => {
            if let Some(m) = motion::from_key(key) {
                run_motion(ed, m);
            } else {
                ed.pending.reset();
            }
        }
    }
}

fn start_char(ed: &mut Editor, c: char) {
    match c {
        '"' => ed.pending.stage = Stage::Register,
        '0' if ed.pending.count.is_none() => run_motion(ed, Motion::LineStart),
        '0'..='9' => push_count(ed, c),
        'd' => ed.pending.stage = Stage::Operator(Operator::Delete),
        'c' => ed.pending.stage = Stage::Operator(Operator::Change),
        'y' => ed.pending.stage = Stage::Operator(Operator::Yank),
        'g' => ed.pending.stage = Stage::G,
        'i' => insert::insert_here(ed),
        'a' => insert::append(ed),
        'A' => insert::append_eol(ed),
        'I' => insert::insert_first_non_blank(ed),
        'o' => insert::open_below(ed),
        'O' => insert::open_above(ed),
        'x' => delete_chars(ed),
        'p' => paste::paste(ed, false),
        'P' => paste::paste(ed, true),
        'Y' => {
            let count = ed.pending.total_count();
            let first = ed.cursor.line;
            let last = first + count - 1;
            operator::apply(ed, Operator::Yank, OpTarget::Lines { first, last });
        }
        'u' => {
            ed.undo();
            ed.pending.reset();
        }
        'v' => visual::enter(ed, false),
        'V' => visual::enter(ed, true),
        '/' => search::begin(ed),
        'n' => search::repeat(ed, true),
        'N' => search::repeat(ed, false),
        _ => {
            if let Some(m) = motion::from_key(Key::Char(c)) {
                run_motion(ed, m);
            } else {
                ed.pending.reset();
            }
        }
    }
}

fn register(ed: &mut Editor, key: Key) {
    if let Key::Char(c) = key
        && c.is_ascii_lowercase()
    {
        ed.pending.register = Some(c);
        ed.pending.stage = Stage::Start;
    } else {
        ed.pending.reset();
    }
}

fn g_prefix(ed: &mut Editor, key: Key) {
    if key == Key::Char('g') {
        run_motion(ed, Motion::FirstLine);
    } else {
        ed.pending.reset();
    }
}

fn operator_stage(ed: &mut Editor, op: Operator, key: Key) {
    let Key::Char(c) = key else {
        ed.pending.reset();
        return;
    };
    match c {
        '0' if ed.pending.op_count.is_none() => apply_motion_op(ed, op, Motion::LineStart),
        '0'..='9' => Pending::push_digit_char(&mut ed.pending.op_count, c),
        'i' => ed.pending.stage = Stage::OperatorObject(op, false),
        'a' => ed.pending.stage = Stage::OperatorObject(op, true),
        'g' => ed.pending.stage = Stage::OperatorG(op),
        'w' if op == Operator::Change => {
            if let Some(target) = motion::change_word_target(ed, ed.pending.total_count()) {
                operator::apply(ed, op, target);
            } else {
                ed.pending.reset();
            }
        }
        c if c == op.key() => {
            let first = ed.cursor.line;
            let last = first + ed.pending.total_count() - 1;
            operator::apply(ed, op, OpTarget::Lines { first, last });
        }
        _ => {
            if let Some(m) = motion::from_key(Key::Char(c)) {
                apply_motion_op(ed, op, m);
            } else {
                ed.pending.reset();
            }
        }
    }
}

fn operator_g(ed: &mut Editor, op: Operator, key: Key) {
    if key == Key::Char('g') {
        apply_motion_op(ed, op, Motion::FirstLine);
    } else {
        ed.pending.reset();
    }
}

fn object_stage(ed: &mut Editor, op: Operator, around: bool, key: Key) {
    if let Key::Char(c) = key
        && let Some(target) = text_object::target(ed, c, around)
    {
        operator::apply(ed, op, target);
    } else {
        ed.pending.reset();
    }
}

fn run_motion(ed: &mut Editor, m: Motion) {
    let count = ed.pending.count.take();
    motion::movement(ed, m, count);
    ed.pending.reset();
}

fn push_count(ed: &mut Editor, c: char) {
    Pending::push_digit_char(&mut ed.pending.count, c);
}

fn apply_motion_op(ed: &mut Editor, op: Operator, m: Motion) {
    if let Some(target) = motion::op_target(ed, m, ed.pending.total_count(), ed.pending.has_count())
    {
        operator::apply(ed, op, target);
    } else {
        ed.pending.reset();
    }
}

fn delete_chars(ed: &mut Editor) {
    let count = ed.pending.total_count();
    let remaining = ed.buffer.line_len(ed.cursor.line) - ed.cursor.col;
    let take = count.min(remaining);
    if take == 0 {
        ed.pending.reset();
        return;
    }
    let idx = ed.cursor_index();
    operator::apply(ed, Operator::Delete, OpTarget::Chars(idx..idx + take));
}
