use std::ops::Range;

use crate::core::buffer::Buffer;
use crate::core::position::Position;
use crate::vim::desired_col::DesiredCol;
use crate::vim::editor::Editor;
use crate::vim::key::Key;
use crate::vim::word;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motion {
    Left,
    Right,
    Down,
    Up,
    LineStart,
    LineEnd,
    WordForward,
    WordBack,
    WordEnd,
    FirstLine,
    LastLine,
}

pub enum OpTarget {
    Chars(Range<usize>),
    Lines {
        first: usize,
        last: usize,
    },
    Block {
        first: usize,
        last: usize,
        left: usize,
        right: usize,
    },
}

pub fn from_key(key: Key) -> Option<Motion> {
    match key {
        Key::Char('h') | Key::Left => Some(Motion::Left),
        Key::Char('l') | Key::Right => Some(Motion::Right),
        Key::Char('j') | Key::Down => Some(Motion::Down),
        Key::Char('k') | Key::Up => Some(Motion::Up),
        Key::Char('$') => Some(Motion::LineEnd),
        Key::Char('w') => Some(Motion::WordForward),
        Key::Char('b') => Some(Motion::WordBack),
        Key::Char('e') => Some(Motion::WordEnd),
        Key::Char('G') => Some(Motion::LastLine),
        _ => None,
    }
}

pub fn movement(ed: &mut Editor, motion: Motion, count: Option<usize>) {
    let n = count.unwrap_or(1);
    let cur = ed.cursor;
    match motion {
        Motion::Left => set_col(ed, cur.col.saturating_sub(n)),
        Motion::Right => set_col(ed, (cur.col + n).min(ed.buffer.line_end_col(cur.line))),
        Motion::Down => vertical(ed, (cur.line + n).min(ed.buffer.line_count() - 1)),
        Motion::Up => vertical(ed, cur.line.saturating_sub(n)),
        Motion::LineStart => set_col(ed, 0),
        Motion::LineEnd => {
            ed.cursor.col = ed.buffer.line_end_col(cur.line);
            ed.desired = DesiredCol::End;
        }
        Motion::WordForward => word_move(ed, n, word::next_word_start),
        Motion::WordEnd => word_move(ed, n, word::next_word_end),
        Motion::WordBack => word_move(ed, n, word::prev_word_start),
        Motion::FirstLine => goto_line(ed, count.map_or(0, |c| c - 1)),
        Motion::LastLine => goto_line(ed, count.map_or(ed.buffer.line_count() - 1, |c| c - 1)),
    }
}

fn set_col(ed: &mut Editor, col: usize) {
    ed.cursor.col = col;
    ed.desired = DesiredCol::Col(col);
}

fn vertical(ed: &mut Editor, line: usize) {
    ed.cursor.line = line;
    ed.cursor.col = match ed.desired {
        DesiredCol::End => ed.buffer.line_end_col(line),
        DesiredCol::Col(c) => c.min(ed.buffer.line_end_col(line)),
    };
}

fn word_move(ed: &mut Editor, n: usize, step: fn(&Buffer, usize) -> usize) {
    let mut idx = ed.cursor_index();
    for _ in 0..n {
        idx = step(&ed.buffer, idx);
    }
    let pos = ed.buffer.clamp_normal(ed.buffer.char_to_pos(idx));
    ed.cursor = pos;
    ed.desired = DesiredCol::Col(pos.col);
}

fn goto_line(ed: &mut Editor, line: usize) {
    let line = line.min(ed.buffer.line_count() - 1);
    ed.cursor = Position::new(line, ed.buffer.first_non_blank(line));
    ed.desired = DesiredCol::Col(ed.cursor.col);
}

pub fn op_target(ed: &Editor, motion: Motion, total: usize, had_count: bool) -> Option<OpTarget> {
    let buffer = &ed.buffer;
    let cur = ed.cursor;
    let idx = ed.cursor_index();
    let last_line = buffer.line_count() - 1;
    let target = match motion {
        Motion::Left => OpTarget::Chars(idx - total.min(cur.col)..idx),
        Motion::Right => OpTarget::Chars(idx..idx + total.min(buffer.line_len(cur.line) - cur.col)),
        Motion::LineStart => OpTarget::Chars(buffer.line_start(cur.line)..idx),
        Motion::LineEnd => {
            OpTarget::Chars(idx..buffer.line_start(cur.line) + buffer.line_len(cur.line))
        }
        Motion::WordForward => OpTarget::Chars(word_forward_range(buffer, idx, total)),
        Motion::WordEnd => {
            let mut j = idx;
            for _ in 0..total {
                j = word::next_word_end(buffer, j);
            }
            OpTarget::Chars(idx..(j + 1).min(buffer.len_chars()))
        }
        Motion::WordBack => {
            let mut j = idx;
            for _ in 0..total {
                j = word::prev_word_start(buffer, j);
            }
            OpTarget::Chars(j..idx)
        }
        Motion::Down => {
            if cur.line == last_line {
                return None;
            }
            OpTarget::Lines {
                first: cur.line,
                last: (cur.line + total).min(last_line),
            }
        }
        Motion::Up => {
            if cur.line == 0 {
                return None;
            }
            OpTarget::Lines {
                first: cur.line.saturating_sub(total),
                last: cur.line,
            }
        }
        Motion::FirstLine => line_target(cur.line, if had_count { total - 1 } else { 0 }),
        Motion::LastLine => line_target(
            cur.line,
            if had_count { total - 1 } else { last_line }.min(last_line),
        ),
    };
    Some(target)
}

fn line_target(current: usize, target: usize) -> OpTarget {
    OpTarget::Lines {
        first: current.min(target),
        last: current.max(target),
    }
}

fn word_forward_range(buffer: &Buffer, idx: usize, count: usize) -> Range<usize> {
    let mut prev = idx;
    let mut end = idx;
    for _ in 0..count {
        prev = end;
        end = word::next_word_start(buffer, end);
    }
    let prev_line = buffer.char_to_pos(prev).line;
    if buffer.char_to_pos(end).line > prev_line {
        let eol = buffer.line_start(prev_line) + buffer.line_len(prev_line);
        end = end.min(eol);
    }
    idx..end.max(idx)
}

pub fn change_word_target(ed: &Editor, total: usize) -> Option<OpTarget> {
    let idx = ed.cursor_index();
    if ed.buffer.char_at(idx).is_none_or(|c| c.is_whitespace()) {
        return op_target(ed, Motion::WordForward, total, false);
    }
    if total == 1
        && let Some((_, end)) = word::word_run(&ed.buffer, idx)
    {
        return Some(OpTarget::Chars(idx..end + 1));
    }
    let mut j = idx;
    for _ in 0..total {
        j = word::next_word_end(&ed.buffer, j);
    }
    Some(OpTarget::Chars(idx..(j + 1).min(ed.buffer.len_chars())))
}
